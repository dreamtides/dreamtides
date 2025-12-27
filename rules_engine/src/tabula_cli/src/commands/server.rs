use std::net::IpAddr;

use anyhow::Result;

use crate::server::server_config;
use crate::server::server_config::ServerConfig;

pub fn server(host: IpAddr, port: u16, max_payload_bytes: usize, once: bool) -> Result<()> {
    server_config::run(ServerConfig { host, port, max_payload_bytes, once })
}
