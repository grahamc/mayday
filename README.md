Dump SOS output from all your Packet servers.

get a token via Vault or a cli arg
using that, every minute enumerate the Packet api for devices
for each device, spawn a ssh process and pipe the output to a file...
see if they're still known server IDs -> respawn, otherwise exit.

Example:

    cargo run -- \
        --output-dir ./output \
        --vault-server http://127.0.0.1:8200 \
        --vault-token-file ~/.vault-token \
        --vault-key packet/creds/xxxxx \
        --packet-project-id xxxxx


using the vault plugin from t0mk.
