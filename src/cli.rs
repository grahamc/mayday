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

    #[structopt(long = "vault-secret-engine")]
    pub vault_secret_engine: String,

    #[structopt(long = "vault-role")]
    pub vault_role: String,

    #[structopt(long = "packet-project-id")]
    pub project_id: String,
}
