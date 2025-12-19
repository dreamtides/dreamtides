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
    Builder::new_multi_thread().enable_all().build()?.block_on(http::serve(config))
}

mod http;
mod listener_runner;
mod listeners;
mod model;
mod serialization;
mod server_workbook_snapshot;

pub use listener_runner::{Listener, ListenerContext, ListenerResult};
pub use model::{Request, Response};
pub use server_workbook_snapshot::{FileMetadata, WorkbookSnapshot};
