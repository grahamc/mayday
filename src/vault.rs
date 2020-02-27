use hashicorp_vault::client::error::Error;
use hashicorp_vault::client::{TokenData, TokenOptions, VaultDuration};
use hashicorp_vault::Client;
use serde::de::DeserializeOwned;

pub struct AutoRevokedClient {
    logger: slog::Logger,
    server: String,
    client: Option<Client<TokenData>>,
}

impl AutoRevokedClient {
    pub fn create_app_client(
        logger: slog::Logger,
        server: String,
        initial_token: String,
    ) -> Result<AutoRevokedClient, Error> {
        // 1. Create a root client with the provided token
        info!(logger, "Creating a root-level Vault client"; "server" => &server);
        let root_client = Client::new(server.clone().as_str(), &initial_token)?;
        // Explicitly drop here to make certain we don't accidentally
        // keep it alive somehow.
        drop(initial_token);

        let app_client = AutoRevokedClient::create(logger, server, &root_client);
        // Drop the root client explicitly to make certain we don't
        // use it ever again.
        drop(root_client);
        app_client
    }

    pub fn create<T>(
        logger: slog::Logger,
        server: String,
        initial_client: &Client<T>,
    ) -> Result<AutoRevokedClient, Error>
    where
        T: DeserializeOwned,
    {
        // 1. Using the initially provided client, create a new token
        //    and drop the initial client.
        let opts = TokenOptions::default().ttl(VaultDuration::hours(24));
        // !!! renew this top level token
        info!(logger, "Creating a sub-token"; "client" => "provided");
        let sub_token_response = initial_client.create_token(&opts)?;
        drop(initial_client);

        // 2. Using this new token, create a new client and return it
        let logger = logger.new(o!(
            "server" => server.clone(),
            "client" => "sub",
        ));
        info!(logger, "Creating a sub-client with the sub-token");
        let sub_client = Client::new(server.clone().as_str(), &sub_token_response.client_token)?;
        // Explicitly drop the sub token since we don't need it from
        // here on. This may change when we start renewing these
        // tokens properly.
        drop(sub_token_response);

        Ok(AutoRevokedClient {
            logger: logger,
            server: server,
            client: Some(sub_client),
        })
    }

    pub fn client(&self) -> &Client<TokenData> {
        &self
            .client
            .as_ref()
            .expect("bug! client should never be None")
    }

    pub fn child_client(&self, name: &'static str) -> Result<AutoRevokedClient, Error> {
        info!(self.logger, "Creating a sub-token for {}", name);

        let logger = self.logger.new(o!(
            "token" => name,
        ));
        AutoRevokedClient::create(logger, self.server.clone(), self.client())
    }
}

impl Drop for AutoRevokedClient {
    fn drop(&mut self) {
        info!(self.logger, "Revoking the application token");
        match self
            .client
            .take()
            .expect("bug! client is None before Drop")
            .revoke()
        {
            Ok(resp) => {
                info!(self.logger, "Successfully revoked"; "response" => ?resp);
            }
            Err(e) => {
                error!(self.logger, "Failed to revoke"; "error" => ?e);
            }
        }
    }
}
