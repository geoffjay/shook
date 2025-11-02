# GitHub Webhook Setup Guide

This guide explains how to configure Shook to receive webhooks from GitHub repositories and automatically execute commands when pull requests are merged.

## Overview

Shook supports GitHub webhooks using HMAC-SHA256 signature verification for security. When a pull request is merged into the main branch, Shook will:

1. Verify the webhook signature
2. Clone or update the repository
3. Execute configured commands

## Configuration

### 1. Configure Shook

Add your GitHub project to `config.yml`:

```yaml
projects:
  - name: my-github-app
    provider: github
    token: your-secret-key-here  # This will be used for HMAC signature verification
    env:
      APP_DIR: /var/www/myapp
      NODE_ENV: production
    commands:
      - "cd $APP_DIR"
      - "git pull origin main"
      - "npm ci"
      - "npm run build"
      - "pm2 restart myapp"
```

**Important Configuration Notes:**
- `provider: github` is required for GitHub projects
- `token` is the secret used for HMAC-SHA256 signature verification
- Use a strong, random secret (e.g., generate with `openssl rand -hex 32`)
- Environment variables in `env` are available to all commands

### 2. Configure GitHub Repository

1. Navigate to your GitHub repository
2. Go to **Settings** → **Webhooks**
3. Click **Add webhook**
4. Configure the webhook:

   - **Payload URL**: `http://your-server:5000/webhook/my-github-app`
     - Replace `your-server:5000` with your actual server address
     - Replace `my-github-app` with the project name from your config

   - **Content type**: Select `application/json`

   - **Secret**: Enter the same secret you used in the `token` field in config.yml

   - **SSL verification**: Enable if using HTTPS (recommended for production)

   - **Which events would you like to trigger this webhook?**
     - Select "Let me select individual events"
     - Check only "Pull requests"

   - **Active**: Ensure this is checked

5. Click **Add webhook**

### 3. Verify Setup

GitHub will immediately send a `ping` event to verify the webhook URL is accessible. Check the webhook's "Recent Deliveries" tab to ensure it was successful.

## Security Best Practices

### 1. Use Strong Secrets

Generate a strong secret for HMAC verification:

```bash
# Generate a 32-byte random secret
openssl rand -hex 32
```

### 2. Use HTTPS

In production, always use HTTPS to prevent token interception:

```nginx
# Example nginx reverse proxy configuration
server {
    listen 443 ssl;
    server_name webhook.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:5000;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header Host $http_host;
    }
}
```

### 3. Restrict Network Access

Use firewall rules to only allow webhook traffic from GitHub's IP ranges:

```bash
# Example iptables rules (GitHub IP ranges change, check GitHub Meta API)
iptables -A INPUT -p tcp --dport 5000 -s 140.82.112.0/20 -j ACCEPT
iptables -A INPUT -p tcp --dport 5000 -s 143.55.64.0/20 -j ACCEPT
# ... add all GitHub IP ranges
iptables -A INPUT -p tcp --dport 5000 -j DROP
```

Get current GitHub IP ranges:
```bash
curl https://api.github.com/meta | jq '.hooks'
```

### 4. Use Separate Secrets per Project

Never reuse secrets across projects. Each project should have its own unique secret.

## Testing

### Manual Testing with Trigger Endpoint

Test your configuration without creating a real pull request:

```bash
curl "http://localhost:5000/trigger/my-github-app?path=owner/repo&repo=https://github.com/owner/repo.git"
```

### Testing with GitHub CLI

Create a test pull request and merge it:

```bash
# Create a test branch
git checkout -b test-webhook
echo "test" >> test.txt
git add test.txt
git commit -m "Test webhook"
git push origin test-webhook

# Create and merge PR using GitHub CLI
gh pr create --title "Test webhook" --body "Testing webhook integration"
gh pr merge --merge
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized Response**
   - Verify the secret in config.yml matches exactly what's configured in GitHub
   - Check that the `X-Hub-Signature-256` header is being sent

2. **Webhook not triggering**
   - Ensure the PR is being merged into the main branch
   - Check that "Pull requests" events are selected in webhook settings
   - Verify the project name in the URL matches the config

3. **Commands not executing**
   - Check Shook logs: `journalctl -u shook -f`
   - Ensure the user running Shook has permissions to execute commands
   - Verify environment variables are set correctly

### Viewing Logs

Shook outputs structured JSON logs. To view and filter:

```bash
# View all logs
journalctl -u shook

# View only GitHub webhook events
journalctl -u shook | jq 'select(.msg | contains("github"))'

# Follow logs in real-time
journalctl -u shook -f
```

### GitHub Webhook Deliveries

Check recent webhook deliveries in GitHub:

1. Go to Settings → Webhooks
2. Click on your webhook
3. Check "Recent Deliveries" tab
4. Click on a delivery to see request/response details

## Example Deployment Scenarios

### Node.js Application with PM2

```yaml
projects:
  - name: nodejs-app
    provider: github
    token: your-secret-here
    env:
      APP_DIR: /var/www/app
      NODE_ENV: production
    commands:
      - "cd $APP_DIR"
      - "git pull origin main"
      - "npm ci --production"
      - "npm run build"
      - "pm2 reload ecosystem.config.js --update-env"
```

### Docker Compose Application

```yaml
projects:
  - name: docker-app
    provider: github
    token: your-secret-here
    env:
      COMPOSE_DIR: /opt/docker/app
    commands:
      - "cd $COMPOSE_DIR"
      - "git pull origin main"
      - "docker-compose pull"
      - "docker-compose down"
      - "docker-compose up -d"
      - "docker system prune -f"
```

### Static Site with Nginx

```yaml
projects:
  - name: static-site
    provider: github
    token: your-secret-here
    env:
      SITE_DIR: /var/www/html
      BUILD_DIR: /tmp/build
    commands:
      - "cd $BUILD_DIR"
      - "git pull origin main"
      - "npm ci"
      - "npm run build"
      - "rsync -av --delete dist/ $SITE_DIR/"
      - "nginx -s reload"
```

## Advanced Configuration

### Multiple Branches

To deploy from different branches to different environments, create separate projects:

```yaml
projects:
  # Production (main branch)
  - name: app-production
    provider: github
    token: prod-secret
    env:
      ENV: production
      DIR: /var/www/prod
    commands:
      - "cd $DIR && ./deploy.sh"

  # Staging (develop branch)
  - name: app-staging
    provider: github
    token: staging-secret
    env:
      ENV: staging
      DIR: /var/www/staging
    commands:
      - "cd $DIR && ./deploy.sh"
```

Configure separate webhooks in GitHub for each, or modify the code to check branch names.

### Notifications

Add notification commands to inform your team:

```yaml
commands:
  - "curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL -d '{\"text\":\"Deployment started\"}'"
  - "cd /var/www/app && git pull"
  - "npm ci && npm run build"
  - "pm2 reload app"
  - "curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL -d '{\"text\":\"Deployment completed\"}'"
```

## Migration from GitLab

If migrating from GitLab to GitHub:

1. Add `provider: github` to your existing project config
2. Generate a new secret for GitHub (don't reuse GitLab tokens)
3. Update the webhook URL in GitHub settings
4. Test with the trigger endpoint before removing GitLab webhook

Both GitLab and GitHub webhooks can coexist for the same project during migration.