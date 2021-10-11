# Setup

## Tunnel

This isn't necessary if the listening port can be opened to the outside world,
or if the GitLab instance is running on the same network as the service.

```shell
ngrok http -subdomain=shook 5000
```

### Run with `systemd`

The `ngrok.yml` configuration requires that the value for `authtoken` be taken
from the account to use and set correctly.

```shell
sudo mkdir -p /etc/ngrok
cat <<EOF | sudo tee /etc/ngrok/ngrok.yml
authtoken: <add_your_token_here>
tunnels:
  shook-http:
    addr: 5000
    proto: http
    subdomain: shook
    auth: "user:secretpassword"
    bind_tls: false
EOF
cat <<EOF | sudo tee /etc/systemd/system/ngrok.service
[Unit]
Description=ngrok
After=network.target

[Service]
ExecStart=/usr/bin/ngrok start --all --config /etc/ngrok/ngrok.yml
ExecReload=/bin/kill -HUP $MAINPID
KillMode=process
IgnoreSIGPIPE=true
Restart=always
RestartSec=3
Type=simple

[Install]
WantedBy=multi-user.target
EOF
sudo chmod 644 /etc/systemd/system/ngrok.service
sudo systemctl enable ngrok.service
sudo systemctl start ngrok.service
```

## Create Webhook

Go to the webhooks preferences for a project, eg.
https://gitlab.com/geoff.jay/shook/-/hooks, and enter:

- the host address given by `ngrok` or a local network IP
- a secret token that will be passed to the service with `--token=<secret>`
- check "Merge request events", uncheck all other triggers
- uncheck "Enable SSL verification"

## Run `shook` with `systemd`

```shell
cat <<EOF | sudo tee /etc/systemd/system/shook.service
[Unit]
Description=Shook Webhook Service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
ExecStart=/usr/local/bin/shook

[Install]
WantedBy=multi-user.target
EOF
sudo chmod 644 /etc/systemd/system/shook.service
sudo systemctl enable shook.service
sudo systemctl start shook.service
```
