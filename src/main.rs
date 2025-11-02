#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_scope;

mod cmd;
mod config;
mod webhook;

use actix_slog::StructuredLogger;
use actix_web::{
    error, get, http::header::HeaderMap, post, web, App, Error, HttpRequest, HttpResponse,
    HttpServer,
};
use async_std::task;
use chrono::prelude::*;
use futures::StreamExt;
use serde::Deserialize;
use slog::Drain;

use cmd::ShookArgs;
use config::{Config, Provider};
use webhook::gitlab::Webhook as GitLabWebhook;
use webhook::github::{self, Webhook as GitHubWebhook};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Deserialize)]
struct TriggerInfo {
    path: String,
    repo: String,
}

fn verify_gitlab(headers: &HeaderMap, token: &str) -> bool {
    match headers.get("X-Gitlab-Token") {
        Some(value) => value.to_str().unwrap() == token,
        None => false,
    }
}

fn verify_github(headers: &HeaderMap, secret: &str, body: &[u8]) -> bool {
    match headers.get("X-Hub-Signature-256") {
        Some(value) => {
            match value.to_str() {
                Ok(signature) => github::verify_signature(secret, body, signature),
                Err(_) => false,
            }
        }
        None => false,
    }
}

#[post("/webhook/{project_name}")]
async fn webhook_handler(
    data: web::Data<Config>,
    req: HttpRequest,
    web::Path(project_name): web::Path<String>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let log = slog_scope::logger();
    let project = data.get_project(project_name.clone()).unwrap();

    // Read the body first
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // Handle based on provider type
    match project.provider {
        Provider::GitLab => {
            if !verify_gitlab(req.headers(), &project.token) {
                warn!(log, "X-Gitlab-Token header verification failed");
                return Ok(HttpResponse::Unauthorized().into());
            }

            debug!(log, "X-Gitlab-Token header verified");
            let webhook = serde_json::from_slice::<GitLabWebhook>(&body)?;
            webhook.dump();

            if config::should_deploy(
                webhook.target_branch(),
                webhook.action(),
                webhook.merge_status(),
            ) {
                match webhook.clone_repository() {
                    Ok(repo_path) => debug!(log, "cloned repository"; "path" => repo_path),
                    Err(e) => error!(log, "failed to clone"; "error" => e),
                }
                let project_clone = project.clone();
                task::spawn(async move { data.execute_commands(project_clone).await });
            }
        }
        Provider::GitHub => {
            if !verify_github(req.headers(), &project.token, &body) {
                warn!(log, "X-Hub-Signature-256 header verification failed");
                return Ok(HttpResponse::Unauthorized().into());
            }

            debug!(log, "X-Hub-Signature-256 header verified");
            let webhook = serde_json::from_slice::<GitHubWebhook>(&body)?;
            webhook.dump();

            if github::should_deploy_github(
                webhook.action(),
                webhook.is_merged(),
                webhook.target_branch(),
            ) {
                match webhook.clone_repository() {
                    Ok(repo_path) => debug!(log, "cloned repository"; "path" => repo_path),
                    Err(e) => error!(log, "failed to clone"; "error" => e),
                }
                let project_clone = project.clone();
                task::spawn(async move { data.execute_commands(project_clone).await });
            }
        }
    }

    Ok(HttpResponse::Ok().into())
}

#[get("/trigger/{project_name}")]
async fn trigger(
    data: web::Data<Config>,
    web::Path(project_name): web::Path<String>,
    info: web::Query<TriggerInfo>,
) -> Result<HttpResponse, Error> {
    let log = slog_scope::logger();
    let project = data.get_project(project_name.clone()).unwrap();
    debug!(log, "trigger project"; "project" => project_name.clone(), "repo" => info.repo.clone(), "provider" => format!("{:?}", project.provider));

    match project.provider {
        Provider::GitLab => {
            let input = format!(
                r#"{{
                    "event_type": "merge_request",
                    "project": {{
                        "default_branch": "main",
                        "git_http_url": "{}",
                        "path_with_namespace": "{}"
                    }},
                    "repository": {{
                        "url": "{}"
                    }},
                    "object_attributes": {{
                        "action": "merge",
                        "target_branch": "main",
                        "source_branch": "staging",
                        "state": "merge",
                        "merge_status": "merged"
                    }}
                }}"#,
                info.repo.clone(),
                info.path.clone(),
                info.repo.clone()
            );
            let webhook = serde_json::from_str::<GitLabWebhook>(&input).unwrap();
            webhook.dump();

            if config::should_deploy(
                webhook.target_branch(),
                webhook.action(),
                webhook.merge_status(),
            ) {
                debug!(log, "handle deployment"; "project" => project_name);
                match webhook.clone_repository() {
                    Ok(repo_path) => debug!(log, "cloned repository"; "path" => repo_path),
                    Err(e) => error!(log, "failed to clone"; "error" => e),
                }
                let project_clone = project.clone();
                task::spawn(async move { data.execute_commands(project_clone).await });
                Ok(HttpResponse::Ok().into())
            } else {
                Ok(HttpResponse::InternalServerError().into())
            }
        }
        Provider::GitHub => {
            // Extract repo name from path (last part)
            let repo_name = info.path.split('/').last().unwrap_or(&info.path);
            let input = format!(
                r#"{{
                    "action": "closed",
                    "repository": {{
                        "name": "{}",
                        "full_name": "{}",
                        "clone_url": "{}",
                        "ssh_url": "{}",
                        "default_branch": "main"
                    }},
                    "pull_request": {{
                        "number": 999,
                        "state": "closed",
                        "title": "Test PR",
                        "merged": true,
                        "merged_at": "2024-01-01T00:00:00Z",
                        "head": {{
                            "ref": "feature-branch",
                            "sha": "abc123"
                        }},
                        "base": {{
                            "ref": "main",
                            "sha": "def456"
                        }}
                    }},
                    "sender": {{
                        "login": "test-trigger"
                    }}
                }}"#,
                repo_name,
                info.path.clone(),
                info.repo.clone(),
                info.repo.clone()
            );
            let webhook = serde_json::from_str::<GitHubWebhook>(&input).unwrap();
            webhook.dump();

            if github::should_deploy_github(
                webhook.action(),
                webhook.is_merged(),
                webhook.target_branch(),
            ) {
                debug!(log, "handle deployment"; "project" => project_name);
                match webhook.clone_repository() {
                    Ok(repo_path) => debug!(log, "cloned repository"; "path" => repo_path),
                    Err(e) => error!(log, "failed to clone"; "error" => e),
                }
                let project_clone = project.clone();
                task::spawn(async move { data.execute_commands(project_clone).await });
                Ok(HttpResponse::Ok().into())
            } else {
                Ok(HttpResponse::InternalServerError().into())
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shook = ShookArgs::new();
    let drain = slog_json::Json::new(std::io::stdout())
        .set_pretty(false)
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, shook.level).fuse();
    let log = slog::Logger::root(drain, o!());
    let _guard = slog_scope::set_global_logger(log);

    let config_file = std::fs::File::open(shook.config)?;
    let config: Config = serde_yaml::from_reader(config_file).unwrap();

    let logger = slog_scope::logger();
    let app_log = logger.new(o!("host" => shook.host.clone(), "port" => shook.port.clone()));
    info!(app_log, "application started"; "started_at" => format!("{}", Utc::now()));

    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .wrap(StructuredLogger::new(
                logger.new(o!("version" => "undefined")),
            ))
            .app_data(config_data.clone())
            .service(webhook_handler)
            .service(trigger)
    })
    .bind(format!("{}:{}", shook.host, shook.port))?
    .run()
    .await
}
