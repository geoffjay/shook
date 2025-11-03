# macOS Setup

This guide covers the setup of Shook webhook service on macOS, including Apple Silicon (M-series) and Intel Macs.

## Installation

### Using Release Binary (Recommended)

Download and install the latest release for Apple Silicon:

```shell
curl -s https://api.github.com/repos/geoffjay/shook/releases/latest \
    | jq '.assets[] | select(.name|test("^shook.*aarch64-apple-darwin.zip$")) | .browser_download_url' \
    | tr -d \" \
    | wget -qi -
unzip $(find . -iname "shook_*_aarch64-apple-darwin.zip")
sudo mv shook /usr/local/bin/
sudo chmod +x /usr/local/bin/shook
```

### Using Homebrew (Alternative)

If you prefer to build from source using Homebrew:

```shell
brew install rust git
git clone https://github.com/geoffjay/shook.git
cd shook
cargo build --release
sudo cp target/release/shook /usr/local/bin/
```

## Configuration

Create the configuration directory and file:

```shell
sudo mkdir -p /usr/local/etc/shook
sudo mkdir -p /usr/local/var/log/shook
sudo mkdir -p /usr/local/var/cache/shook
```

Create the configuration file:

```shell
sudo tee /usr/local/etc/shook/config.yml <<EOF
projects:
  # GitLab project (provider defaults to gitlab)
  - name: my-gitlab-project
    token: your-gitlab-token
    env:
      DEPLOY_DIR: /Users/yourusername/projects/my-app
    commands:
      - "cd \$DEPLOY_DIR"
      - "git pull"
      - "npm install"
      - "npm run build"

  # GitHub project example
  - name: my-github-project
    provider: github
    token: your-github-webhook-secret  # Used for HMAC signature verification
    env:
      DEPLOY_DIR: /Users/yourusername/projects/another-app
    commands:
      - "cd \$DEPLOY_DIR"
      - "docker-compose down"
      - "docker-compose pull"
      - "docker-compose up -d"
EOF
```

## Running with launchd

macOS uses `launchd` instead of systemd for service management. Copy the provided plist file:

```shell
sudo cp service-files/com.geoffjay.shook.plist /Library/LaunchDaemons/
sudo chown root:wheel /Library/LaunchDaemons/com.geoffjay.shook.plist
sudo chmod 644 /Library/LaunchDaemons/com.geoffjay.shook.plist
```

### Load and start the service:

```shell
sudo launchctl load /Library/LaunchDaemons/com.geoffjay.shook.plist
sudo launchctl start com.geoffjay.shook
```

### Check service status:

```shell
sudo launchctl list | grep shook
```

### View logs:

```shell
tail -f /usr/local/var/log/shook/stdout.log
tail -f /usr/local/var/log/shook/stderr.log
```

### Stop and unload the service:

```shell
sudo launchctl stop com.geoffjay.shook
sudo launchctl unload /Library/LaunchDaemons/com.geoffjay.shook.plist
```

## Webhook Setup

### Exposing the Service

If you need to expose the webhook service to the internet, you can use:

1. **ngrok** (recommended for testing):
   ```shell
   brew install ngrok
   ngrok http 5000
   ```

2. **Cloudflare Tunnel**:
   ```shell
   brew install cloudflare/cloudflare/cloudflared
   cloudflared tunnel --url http://localhost:5000
   ```

3. **Port forwarding** on your router (for production)

### GitLab

1. Go to your GitLab project → Settings → Webhooks
2. URL: `http://your-server:5000/webhook/your-project-name`
3. Secret Token: Enter the token from your config
4. Trigger: Select "Merge request events"
5. Click "Add webhook"

### GitHub

1. Go to your GitHub repository → Settings → Webhooks
2. Payload URL: `http://your-server:5000/webhook/your-project-name`
3. Content type: `application/json`
4. Secret: Enter the token from your config (used for HMAC signature)
5. Events: Select "Pull requests"
6. Click "Add webhook"

## Testing

Test your webhook configuration using the trigger endpoint:

```shell
curl "http://localhost:5000/trigger/your-project-name?path=owner/repo&repo=https://github.com/owner/repo.git"
```

## Troubleshooting

### Check if service is running:

```shell
sudo launchctl list | grep shook
ps aux | grep shook
```

### View logs:

```shell
cat /usr/local/var/log/shook/stdout.log
cat /usr/local/var/log/shook/stderr.log
```

### Manually test the binary:

```shell
shook --help
shook --verbose --config /usr/local/etc/shook/config.yml
```

### Permissions issues:

Ensure the cache directory is writable:

```shell
sudo chmod -R 755 /usr/local/var/cache/shook
```

## Security Considerations

- Always use strong, unique tokens/secrets for each project
- Consider using HTTPS in production with a reverse proxy (nginx, caddy)
- Repositories are cloned to `/usr/local/var/cache/shook/` - ensure proper permissions
- Run the service with appropriate user permissions (consider creating a dedicated user)
- Use firewall rules to restrict access to the webhook endpoint
