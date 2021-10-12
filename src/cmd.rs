extern crate clap;

use clap::{App, Arg};
use std::ffi::OsString;

#[derive(Debug, PartialEq)]
pub struct ShookArgs {
    pub port: String,
    pub host: String,
    pub config: String,
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
                Arg::with_name("config")
                    .long("config")
                    .short("c")
                    .takes_value(true)
                    .help("configuration file to load"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .multiple(true)
                    .help("verbose output"),
            )
            .get_matches_from_safe(args)?;

        let port = matches.value_of("port").unwrap_or("5000");
        let host = matches.value_of("host").unwrap_or("0.0.0.0");
        let config = matches.value_of("config").unwrap_or("config.yml");
        let level = match matches.occurrences_of("verbose") {
            0 => slog::Level::Info,
            1 => slog::Level::Debug,
            _ => slog::Level::Trace,
        };

        Ok(ShookArgs {
            port: port.to_string(),
            host: host.to_string(),
            config: config.to_string(),
            level,
        })
    }
}
