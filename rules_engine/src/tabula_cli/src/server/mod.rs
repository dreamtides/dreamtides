use std::net::IpAddr;

use anyhow::Result;
use tokio::runtime::Builder;

pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub max_payload_bytes: usize,
    pub once: bool,
}

pub fn run(config: ServerConfig) -> Result<()> {
    println!("Tabula Server running at {}:{}", config.host, config.port);
    Builder::new_multi_thread().enable_all().build()?.block_on(http::serve(config))
}

mod http;
mod serialization;

pub mod listener_runner;
pub mod listeners;
pub mod model;
pub mod server_workbook_snapshot;
