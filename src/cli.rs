use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "mayday")]
pub struct CliOptions {
    #[structopt(long = "output-dir", parse(from_os_str))]
    pub output_dir: PathBuf,

    #[structopt(long = "vault-server")]
    pub vault_server: String,

    #[structopt(long = "vault-token-file", parse(from_os_str))]
    pub vault_token_file: PathBuf,

    #[structopt(long = "vault-key")]
    pub vault_key: String,

    #[structopt(long = "packet-project-id")]
    pub project_id: String,
}
