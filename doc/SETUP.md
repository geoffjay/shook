# Setup

This guide covers the setup of Shook webhook service for both GitLab and GitHub.
For detailed GitHub-specific setup instructions, see [GITHUB_SETUP.md](GITHUB_SETUP.md).

## Tunnel

This isn't necessary if the listening port can be opened to the outside world,
or if the GitLab/GitHub instance is running on the same network as the service.

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

### GitLab

Go to the webhooks preferences for a project, eg.
https://gitlab.com/geoff.jay/shook/-/hooks, and enter:

- the host address given by `ngrok` or a local network IP
- a secret token that will be configured with the project definition
- check "Merge request events", uncheck all other triggers
- uncheck "Enable SSL verification"

### GitHub

See [GITHUB_SETUP.md](GITHUB_SETUP.md) for detailed GitHub webhook configuration instructions.

## Configure

```shell
sudo mkdir /etc/shook/
cat <<EOF | sudo tee /etc/shook/config.yml
projects:
  # GitLab project (provider defaults to gitlab)
  - name: sample
    token: really-gud-secret
    env:
      LOG: /tmp/sample.log
    commands:
      - "touch $LOG"
      - "echo test >> $LOG"

  # GitHub project example
  - name: github-sample
    provider: github
    token: github-webhook-secret
    env:
      LOG: /tmp/github.log
    commands:
      - "touch $LOG"
      - "echo github test >> $LOG"
EOF
```

## Run `shook` with `systemd`

A reference systemd service file is provided in `service-files/shook.service`. To install:

```shell
sudo cp service-files/shook.service /etc/systemd/system/
sudo chmod 644 /etc/systemd/system/shook.service
sudo systemctl daemon-reload
sudo systemctl enable shook.service
sudo systemctl start shook.service
```

Check the service status:

```shell
sudo systemctl status shook.service
```

View logs:

```shell
sudo journalctl -u shook.service -f
```

## macOS Setup

For macOS installation and launchd setup, see [MACOS_SETUP.md](MACOS_SETUP.md).
