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
use config::Config;
use webhook::gitlab::Webhook;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Deserialize)]
struct TriggerInfo {
    path: String,
    repo: String,
}

fn verify(headers: &HeaderMap, state: &str) -> bool {
    match headers.get("X-Gitlab-Token") {
        Some(value) => value.to_str().unwrap() == state,
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

    if verify(req.headers(), &project.token) {
        debug!(log, "X-Gitlab-Token header verified");

        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(error::ErrorBadRequest("overflow"));
            }
            body.extend_from_slice(&chunk);
        }

        let webhook = serde_json::from_slice::<Webhook>(&body)?;
        webhook.dump();

        if config::should_deploy(
            webhook.target_branch(),
            webhook.action(),
            webhook.merge_status(),
        ) {
            match webhook.clone_repository() {
                Ok(project) => debug!(log, "cloned repository"; "project" => project),
                Err(e) => error!(log, "failed to clone"; "error" => e),
            }
            task::spawn(async move { data.execute_commands(project).await });
        }

        Ok(HttpResponse::Ok().into())
    } else {
        warn!(log, "X-Gitlab-Token header verification failed");
        Ok(HttpResponse::Unauthorized().into())
    }
}

#[get("/trigger/{project_name}")]
async fn trigger(
    data: web::Data<Config>,
    web::Path(project_name): web::Path<String>,
    info: web::Query<TriggerInfo>,
) -> Result<HttpResponse, Error> {
    let log = slog_scope::logger();
    let project = data.get_project(project_name.clone()).unwrap();
    debug!(log, "trigger project"; "project" => project_name.clone(), "repo" => info.repo.clone());
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
    let webhook = serde_json::from_str::<Webhook>(&input).unwrap();
    webhook.dump();

    if config::should_deploy(
        webhook.target_branch(),
        webhook.action(),
        webhook.merge_status(),
    ) {
        debug!(log, "handle deployment"; "project" => project_name);
        match webhook.clone_repository() {
            Ok(project) => debug!(log, "cloned repository"; "{}" => project),
            Err(e) => error!(log, "failed to clone"; "{}" => e),
        }
        task::spawn(async move { data.execute_commands(project).await });
        Ok(HttpResponse::Ok().into())
    } else {
        Ok(HttpResponse::InternalServerError().into())
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
