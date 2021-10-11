#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_scope;

mod cmd;
mod config;
mod webhook;

use actix_web::{
    error, http::header::HeaderMap, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use async_std::task;
use chrono::prelude::*;
use futures::StreamExt;
use slog::Drain;

use cmd::ShookArgs;
use config::Config;
use webhook::Webhook;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

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

        trace!(log, "webhook data"; "event_type" => webhook.event_type());
        trace!(log, "webhook data"; "repository_url" => webhook.repository_url());
        trace!(log, "webhook data"; "action" => webhook.action());
        trace!(log, "webhook data"; "target_branch" => webhook.target_branch());
        trace!(log, "webhook data"; "source_branch" => webhook.source_branch());
        trace!(log, "webhook data"; "state" => webhook.state());
        trace!(log, "webhook data"; "merge_status" => webhook.merge_status());

        task::spawn(async move { data.execute_commands(project.to_string()).await });

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
        .set_pretty(false)
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, shook.level).fuse();
    let log = slog::Logger::root(drain, o!());
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
