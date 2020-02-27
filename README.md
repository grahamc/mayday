# Dump SOS output from all your Packet servers.

1. get a Packet API token via Vault
2. using that, every 20s,  enumerate the Packet api for devices
   for each device, spawn a ssh process and pipe the output to a
   file...
3. respawn any failed connections

Every few seconds, it sends a backspace character (`\b` to the console.
This is to detect failed connections, though there is probably a better
way. First I tried `\n`, but ipmitool seems to interpret this as a
`\r\n`, and the `\r` is equivalent to the Ctrl-M some servers expect
to enter the firmware interface.

## Example:

    cargo run -- \
        --output-dir ./output \
        --vault-server http://127.0.0.1:8200 \
        --vault-token-file ~/.vault-token \
        --vault-secret-engine packet \
        --vault-role 1h-read-only-user \
        --packet-project-id your-packet-project-id

after having configured your Vault server to use this plugin:
https://github.com/packethost/vault-plugin-secrets-packet

Locally, the 1h-read-only-user role was created with:

    $ vault kv put packet/role/1h-read-only-user \
        type=user read_only=true ttl=30 max_ttl=3600

It should work equally well with a project token.
