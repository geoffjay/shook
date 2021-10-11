extern crate clap;

use clap::{App, Arg};
use std::ffi::OsString;

#[derive(Debug, PartialEq)]
pub struct ShookArgs {
    pub token: String,
    pub port: String,
    pub host: String,
    pub level: slog::Level,
}

impl ShookArgs {
    pub fn new() -> Self {
        Self::new_from(std::env::args_os()).unwrap_or_else(|e| e.exit())
    }

    fn new_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = App::new("shook")
            .version("0.1")
            .about("Gitlab Webhook Handler")
            .author("Geoff Johnson <geoff.jay@gmail.com>")
            .arg(
                Arg::with_name("token")
                    .long("token")
                    .short("t")
                    .takes_value(true)
                    .help("X-Gitlab-Token")
                    .required(true),
            )
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .short("p")
                    .takes_value(true)
                    .help("port to listen on"),
            )
            .arg(
                Arg::with_name("host")
                    .long("host")
                    .short("h")
                    .takes_value(true)
                    .help("host to bind to"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .multiple(true)
                    .help("verbose output"),
            )
            .get_matches_from_safe(args)?;

        let token = matches
            .value_of("token")
            .expect("A Webhook token is required");
        let port = matches.value_of("port").unwrap_or("5000");
        let host = matches.value_of("host").unwrap_or("0.0.0.0");
        let level = match matches.occurrences_of("verbose") {
            0 => slog::Level::Info,
            1 => slog::Level::Debug,
            _ => slog::Level::Trace,
        };

        Ok(ShookArgs {
            token: token.to_string(),
            port: port.to_string(),
            host: host.to_string(),
            level,
        })
    }
}
