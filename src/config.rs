use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// Gateway base URL, for example http://127.0.0.1:9765.
    #[arg(
        long,
        env = "CANVAS_BRIDGE_GATEWAY",
        default_value = "http://127.0.0.1:9765"
    )]
    pub gateway: String,

    /// Address where canvas-bridge listens.
    #[arg(long, env = "CANVAS_BRIDGE_BIND", default_value = "127.0.0.1:9876")]
    pub bind: SocketAddr,
}

impl Config {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
