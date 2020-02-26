Dump SOS output from all your Packet servers.

get a token via Vault or a cli arg
using that, every minute enumerate the Packet api for devices
for each device, spawn a ssh process and pipe the output to a file...
see if they're still known server IDs -> respawn, otherwise exit.
