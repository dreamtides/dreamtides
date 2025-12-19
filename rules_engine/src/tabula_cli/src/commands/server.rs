use std::net::IpAddr;

use anyhow::Result;

use crate::server;
use crate::server::ServerConfig;

pub fn server(host: IpAddr, port: u16, max_payload_bytes: usize, once: bool) -> Result<()> {
    server::run(ServerConfig { host, port, max_payload_bytes, once })
}
