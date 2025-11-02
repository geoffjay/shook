[![release status](https://github.com/geoffjay/shook/actions/workflows/release.yml/badge.svg)](https://github.com/geoffjay/shook/actions?query=workflow%3A%22release%22)

# Shook

Lightweight webhook server to listen to GitLab and GitHub webhooks and execute configured commands. Perfect for automated deployments, CI/CD pipelines, and custom automation tasks.

## Features

- **Multi-Provider Support**: Works with both GitLab and GitHub webhooks
- **Secure**: Token-based authentication for GitLab, HMAC-SHA256 signature verification for GitHub
- **Automated Git Operations**: Automatically clones and updates repositories
- **Custom Commands**: Execute any shell commands in response to webhook events
- **Environment Variables**: Pass custom environment variables to commands
- **Testing Endpoint**: Manual trigger endpoint for testing deployments
- **Structured Logging**: JSON-formatted logs for easy parsing and monitoring

## Supported Events

- **GitLab**: Merge requests merged to main branch
- **GitHub**: Pull requests merged to main branch

## Install

```shell
curl -s https://api.github.com/repos/geoffjay/shook/releases/latest \
    | jq '.assets[] | select(.name|test("^shook.*linux-musl.zip$")) | .browser_download_url' \
    | tr -d \" \
    | wget -qi -
unzip $(find . -iname "shook_*.zip")
sudo mv shook /usr/local/bin/
```

Check the [setup](doc/SETUP.md) documentation for any remaining steps.

## Configuration

Create a `config.yml` file with your projects:

```yaml
projects:
  # GitLab project (provider defaults to gitlab if not specified)
  - name: my-gitlab-project
    token: your-gitlab-token
    env:
      DEPLOY_DIR: /var/www/app
    commands:
      - "cd $DEPLOY_DIR"
      - "git pull"
      - "npm install"
      - "npm run build"

  # GitHub project
  - name: my-github-project
    provider: github
    token: your-github-webhook-secret  # Used for HMAC signature verification
    env:
      DEPLOY_DIR: /var/www/another-app
    commands:
      - "cd $DEPLOY_DIR"
      - "docker-compose down"
      - "docker-compose pull"
      - "docker-compose up -d"
```

## Usage

### Starting the Server

```shell
# Default (port 5000, host 0.0.0.0)
shook

# Custom port and host
shook --port 8080 --host 127.0.0.1

# Custom config file
shook --config /path/to/config.yml

# Verbose logging
shook --verbose
```

### Webhook URLs

Configure your repository webhooks to point to:
- `http://your-server:5000/webhook/{project_name}`

Where `{project_name}` matches the name in your config file.

### Testing

Test your webhook configuration using the trigger endpoint:
- `http://your-server:5000/trigger/{project_name}?path=owner/repo&repo=https://github.com/owner/repo.git`

## Webhook Setup

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

## Security Considerations

- **GitLab**: Uses token-based authentication via `X-Gitlab-Token` header
- **GitHub**: Uses HMAC-SHA256 signature verification via `X-Hub-Signature-256` header
- Always use strong, unique tokens/secrets for each project
- Consider using HTTPS in production
- Repositories are cloned to `/var/cache/shook/` - ensure proper permissions

## Develop

### Build

```shell
cargo build
```

### Execute

```shell
cargo run
```

To see all arguments that are available execute the command `cargo run -- --help`.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
