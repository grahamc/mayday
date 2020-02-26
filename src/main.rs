extern crate serde;

#[macro_use]
extern crate slog;
extern crate reqwest;
use std::fs::File;
use std::io::Read;

use structopt::StructOpt;
mod cli;
mod logging;
mod servers;
mod tailer;

use std::{thread, time};

fn main() {
    let logger = crate::logging::root();
    info!(logger, "MAYDAY MAYDAY MAYDAY");
    let opt = crate::cli::CliOptions::from_args();
    info!(logger, "Writing output to {:?}", opt.output_dir);

    let mut token = String::new();

    info!(
        logger,
        "Reading Vault token from {:?}", opt.vault_token_file
    );
    File::open(opt.vault_token_file)
        .expect("Failed to open Vault token file")
        .read_to_string(&mut token)
        .expect("Failed to read token");
    let client = hashicorp_vault::Client::new(opt.vault_server.as_str(), token).unwrap();

    info!(logger, "Creating a new temporary client token...");
    let opts = hashicorp_vault::client::TokenOptions::default()
        .ttl(hashicorp_vault::client::VaultDuration::hours(24));
    // todo: renew this top level token
    let res = client
        .create_token(&opts)
        .expect("Failed to create an application specific token");
    let client = hashicorp_vault::Client::new(opt.vault_server.as_str(), res.client_token)
        .expect("Failed to create a new application-specific vault client");

    let mut server_monitor = crate::servers::Monitor::new(
        logger.new(o!()),
        hashicorp_vault::Client::new(
            opt.vault_server.as_str(),
            client
                .create_token(&opts)
                .expect("Failed to create a Packet sub-token")
                .client_token,
        )
        .expect("Failed to create a new temporary vault client"),
        opt.project_id,
        opt.vault_key,
    );

    let mut tailer = crate::tailer::Tailer::new(logger.new(o!()), opt.output_dir);

    loop {
        let diff = server_monitor.sync().expect("Failed to sync!");
        tailer.update(diff.added, diff.removed);

        thread::sleep(time::Duration::from_secs(30));
    }

    info!(logger, "Revoking temporary app token and all of its tokens");
    client.revoke().unwrap();
}
