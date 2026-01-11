use anyhow::{Context, Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

use crate::logging::writer::SizeRotatingWriter;

pub fn init_logging(verbose: bool) -> Result<()> {
    let log_dir = crate::config::get_llmc_root().join("logs");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;
    let json_log_file = log_dir.join("llmc.jsonl");
    let json_writer = SizeRotatingWriter::new(json_log_file)?;
    let json_layer = fmt::layer()
        .with_writer(json_writer)
        .with_ansi(false)
        .json()
        .with_current_span(false)
        .with_span_list(false);
    let default_level = if verbose { "info" } else { "warn" };
    let env_filter =
        EnvFilter::try_from_env("LLMC_LOG").unwrap_or_else(|_| EnvFilter::new(default_level));
    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_filter(env_filter);
    tracing_subscriber::registry().with(json_layer).with(stderr_layer).init();
    Ok(())
}
