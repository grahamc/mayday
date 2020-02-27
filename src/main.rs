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
mod vault;

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

    let top_level_client =
        vault::AutoRevokedClient::create_app_client(logger.new(o!()), opt.vault_server, token)
            .expect("Failed to create the top level Vault client");

    let mut server_monitor = crate::servers::Monitor::new(
        logger.new(o!()),
        top_level_client
            .child_client("packet")
            .expect("Failed to create a Packet sub-token"),
        opt.project_id,
        opt.vault_secret_engine,
        opt.vault_role,
    );

    let mut tailer = crate::tailer::Tailer::new(logger.new(o!()), opt.output_dir);

    loop {
        let diff = server_monitor.sync().expect("Failed to sync!");
        tailer.update(diff.added, diff.removed);

        thread::sleep(time::Duration::from_secs(30));
    }
}
