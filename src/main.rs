#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_scope;

mod cmd;
mod webhook;

use actix_web::{
    error, http::header::HeaderMap, web, App, Error, HttpRequest, HttpResponse, HttpServer, post,
};
use chrono::prelude::*;
use futures::StreamExt;
use slog::Drain;

use cmd::ShookArgs;
use webhook::Webhook;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

struct Config {
    token: String,
}

fn verify(headers: &HeaderMap, state: &str) -> bool {
    match headers.get("X-Gitlab-Token") {
        Some(value) => value.to_str().unwrap() == state,
        None => false,
    }
}

#[post("/{project}")]
async fn resp(data: web::Data<Config>, req: HttpRequest, mut payload: web::Payload) -> Result<HttpResponse, Error> {
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

        debug!(log, "request body"; "data" => format!("{:?}", body));
        let webhook = serde_json::from_slice::<Webhook>(&body)?;
        debug!(log, "webhook data"; "event_type" => webhook.event_type());
        debug!(log, "webhook data"; "repository_url" => webhook.repository_url());
        debug!(log, "webhook data"; "action" => webhook.action());
        debug!(log, "webhook data"; "target_branch" => webhook.target_branch());
        debug!(log, "webhook data"; "source_branch" => webhook.source_branch());
        debug!(log, "webhook data"; "state" => webhook.state());
        debug!(log, "webhook data"; "merge_status" => webhook.merge_status());

        Ok(HttpResponse::Ok().finish())
    } else {
        warn!(log, "X-Gitlab-Token header verification failed");
        Ok(HttpResponse::Unauthorized().finish())
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

    let config = Config { token: shook.token };

    let logger = slog_scope::logger();
    let app_log = logger.new(o!("host" => shook.host.clone(), "port" => shook.port.clone()));
    info!(app_log, "application started"; "started_at" => format!("{}", Utc::now()));

    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .service(resp)
    })
    .bind(format!("{}:{}", shook.host, shook.port))?
    .run()
    .await
}
