#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_scope;

mod cmd;
mod webhook;

use actix_web::{
    error, http::header::HeaderMap, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use chrono::prelude::*;
use futures::StreamExt;
use serde::Deserialize;
use slog::Drain;
use std::process::Command;

use cmd::ShookArgs;
use webhook::Webhook;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Deserialize)]
struct Project {
    name: String,
    commands: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(skip)]
    token: String,
    projects: Vec<Project>,
}

impl Config {
    fn execute_commands(&self, project_name: String) {
        let log = slog_scope::logger();
        debug!(log, "run commands for project"; "project_name" => project_name.clone());
        for project in self.projects.iter() {
            if project.name.clone() == project_name.clone() {
                debug!(log, "processor"; "project_name" => project.name.clone());
                for command in project.commands.iter() {
                    debug!(log, "processor"; "command" => command.clone());
                    // will have to iterate commands here, or collect into a script and execute
                    Command::new("echo")
                        .arg("test".to_string())
                        .spawn()
                        .expect("failed");
                }
            }
        }
    }
}

fn verify(headers: &HeaderMap, state: &str) -> bool {
    match headers.get("X-Gitlab-Token") {
        Some(value) => value.to_str().unwrap() == state,
        None => false,
    }
}

#[post("/{project}")]
async fn webhook_handler(
    data: web::Data<Config>,
    req: HttpRequest,
    web::Path(project): web::Path<String>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let log = slog_scope::logger();
    if verify(req.headers(), &data.token) {
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

        debug!(log, "webhook data"; "event_type" => webhook.event_type());
        debug!(log, "webhook data"; "repository_url" => webhook.repository_url());
        debug!(log, "webhook data"; "action" => webhook.action());
        debug!(log, "webhook data"; "target_branch" => webhook.target_branch());
        debug!(log, "webhook data"; "source_branch" => webhook.source_branch());
        debug!(log, "webhook data"; "state" => webhook.state());
        debug!(log, "webhook data"; "merge_status" => webhook.merge_status());

        data.execute_commands(project.to_string());

        Ok(HttpResponse::Ok().into())
    } else {
        warn!(log, "X-Gitlab-Token header verification failed");
        Ok(HttpResponse::Unauthorized().into())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shook = ShookArgs::new();
    let drain = slog_json::Json::new(std::io::stdout())
        .set_pretty(true)
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, shook.level).fuse();
    let log = slog::Logger::root(drain, o!("format" => "pretty"));
    let _guard = slog_scope::set_global_logger(log);

    let config_file = std::fs::File::open(shook.config)?;
    let mut config: Config = serde_yaml::from_reader(config_file).unwrap();
    config.token = shook.token;

    let logger = slog_scope::logger();
    let app_log = logger.new(o!("host" => shook.host.clone(), "port" => shook.port.clone()));
    info!(app_log, "application started"; "started_at" => format!("{}", Utc::now()));

    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .service(webhook_handler)
    })
    .bind(format!("{}:{}", shook.host, shook.port))?
    .run()
    .await
}
