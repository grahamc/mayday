use crate::vault::AutoRevokedClient;
use hashicorp_vault::client::VaultResponse;
use serde::Deserialize;
use std::collections::HashSet;

pub struct Diff<T> {
    pub added: HashSet<T>,
    pub removed: HashSet<T>,
}

struct LeasedKey<K> {
    key: K,
    lease_id: String,
    renewable: bool,
}

#[derive(Deserialize)]
struct PacketKey {
    api_key_token: String,
}

#[derive(Deserialize, Debug)]
struct Devices {
    devices: Vec<Device>,
    meta: PacketMeta,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct Device {
    pub id: String,
    pub hostname: String,
    facility: Facility,
}

impl Device {
    pub fn sos(&self) -> String {
        format!("{}@sos.{}.packet.net", self.id, self.facility.code)
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
struct Facility {
    code: String,
}

#[derive(Deserialize, Debug)]
struct PacketMeta {
    first: PacketLink,
    previous: Option<PacketLink>,
    next: Option<PacketLink>,
    last: PacketLink,
    current_page: usize,
    last_page: usize,
    total: usize,
}

#[derive(Deserialize, Debug)]
struct PacketLink {
    href: String,
}

pub struct Monitor {
    logger: slog::Logger,
    client: AutoRevokedClient,
    secret_engine: String,
    vault_role: String,
    project_id: String,
    token: Option<LeasedKey<PacketKey>>,
    devices: HashSet<Device>,
}

impl Monitor {
    pub fn new(
        logger: slog::Logger,
        client: AutoRevokedClient,
        project_id: String,
        secret_engine: String,
        vault_role: String,
    ) -> Monitor {
        Monitor {
            logger,
            client,
            project_id,
            secret_engine,
            vault_role,
            token: None,
            devices: HashSet::new(),
        }
    }

    pub fn sync(&mut self) -> reqwest::Result<Diff<Device>> {
        let mut upstream_devices: HashSet<Device> = self.fetch_once()?.into_iter().collect();

        std::mem::swap(&mut self.devices, &mut upstream_devices);
        let old_devices = upstream_devices;

        let added: HashSet<Device> = self.devices.difference(&old_devices).cloned().collect();
        let removed: HashSet<Device> = old_devices.difference(&self.devices).cloned().collect();

        return Ok(Diff {
            added: added,
            removed: removed,
        });
    }

    fn fetch_once(&mut self) -> reqwest::Result<Vec<Device>> {
        let logger = self.logger.new(o!());

        let url = format!(
            "https://api.packet.net/projects/{}/devices",
            &self.project_id
        );

        let token = &self
            .get_token()
            .expect("Did not get a Packet token")
            .api_key_token;

        let client = reqwest::Client::new();
        info!(logger, "Requesting devices");
        let devices: Devices = client
            .get(&url)
            .query(&[("per_page", "1000")])
            .header("X-Auth-Token", token)
            .send()?
            .json()
            .expect("Failed deserializing data");
        if devices.meta.last_page > 1 {
            warn!(
                logger,
                "Last page: {} ... but not paginating. Try having fewer than 1,000 servers.",
                devices.meta.last_page
            );
        }

        Ok(devices.devices)
    }

    fn get_token(&mut self) -> Option<&PacketKey> {
        if let Some(token) = self.token.take() {
            if token.renewable {
                info!(self.logger, "Renewing Packet token");
                match self.client.client().renew_lease(&token.lease_id, Some(60)) {
                    Ok(res) => {
                        info!(self.logger, "Renewed Packet token successfully");
                        self.token = Some(LeasedKey {
                            key: token.key,
                            renewable: res.renewable.expect("renewable missing from Vault reply"),
                            lease_id: res.lease_id.expect("lease_id missing from Vault reply"),
                        });
                    }
                    Err(e) => {
                        info!(self.logger, "Failed to renew Packet token"; "err" => %e);
                        self.token = None;
                    }
                }
            }
        }

        if self.token.is_none() {
            info!(self.logger, "Creating a new Packet token");
            let res: VaultResponse<PacketKey> = self
                .client
                .client()
                .get_secret_engine_creds(&self.secret_engine, &self.vault_role)
                .expect("Failed to get a Packet token");

            self.token = Some(LeasedKey {
                key: res.data.expect("Token Data missing from Vault reply"),
                renewable: res.renewable.expect("renewable missing from Vault reply"),
                lease_id: res.lease_id.expect("lease_id missing from Vault reply"),
            });
        }

        self.token.as_ref().map(|ref token| &token.key)
    }
}
