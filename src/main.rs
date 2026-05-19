mod config;
mod gateway;
mod protocol;
mod server;

use anyhow::Result;
use config::Config;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("canvas_bridge=info".parse()?))
        .init();

    server::serve(Config::parse_args()).await
}
