#![warn(rust_2018_idioms)]

#[macro_use]
extern crate slog_scope;

mod auth;
mod db;
mod error;
mod extractors;
mod headers;
mod logging;
mod metrics;
mod middleware;
mod routers;
mod routes;
mod server;
mod settings;
mod tags;

use docopt::Docopt;
use sentry::ClientInitGuard;
use serde::Deserialize;
use std::error::Error;

const USAGE: &str = "
Usage: autoendpoint [options]

Options:
    -h, --help              Show this message
    --config=CONFIGFILE     AutoEndpoint configuration file path.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_config: Option<String>,
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let settings = settings::Settings::with_env_and_config_file(&args.flag_config)?;
    logging::init_logging(!settings.human_logs).expect("Logging failed to initialize");
    debug!("Starting up...");

    // Configure sentry error capture
    let _sentry_guard = configure_sentry();

    // Run server...
    let server = server::Server::with_settings(settings)
        .await
        .expect("Could not start server");
    info!("Server started");
    server.await?;

    // Shutdown
    info!("Server closing");
    logging::reset_logging();
    Ok(())
}

fn configure_sentry() -> ClientInitGuard {
    let options = sentry::ClientOptions {
        release: sentry::release_name!(),
        ..sentry::ClientOptions::default()
    };

    sentry::init(options)
}
