#[macro_use]
extern crate slog;
extern crate slog_json;

use slog::DrainExt;

fn main() {
    println!("Hello, world!");
    let drain = Mutex::new(slog_json::Json::default(std::io::stderr()));
    let root_logger = slog::Logger::root(drain, o!("version" => "0.5"));
    info!(root_logger, "Application started";
        "started_at" => format!("{}", time::now().rfc3339()));
}
