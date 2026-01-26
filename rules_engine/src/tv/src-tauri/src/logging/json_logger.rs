use std::io;
use std::path::PathBuf;

use chrono::Utc;
use chrono_tz::America::Los_Angeles;
use tracing::Level;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};

/// Initializes the tracing subscriber with JSONL or compact output.
pub fn initialize(jsonl: bool) {
    let log_path = log_file_path();
    let writer = FileAndStdout::new(&log_path);
    let filter = build_env_filter();

    if jsonl {
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_timer(PacificTime)
            .with_target(false)
            .with_current_span(false)
            .with_span_list(false)
            .with_writer(writer)
            .flatten_event(true);
        Registry::default().with(filter).with(json_layer).init();
    } else {
        let compact_layer = tracing_subscriber::fmt::layer()
            .with_timer(PacificTime)
            .with_target(false)
            .with_writer(writer)
            .compact();
        Registry::default().with(filter).with(compact_layer).init();
    }

    tracing::info!(
        component = "tv.logging",
        log_file = %log_path.display(),
        jsonl_mode = jsonl,
        "Logging initialized"
    );
}

/// Returns the path for today's log file.
pub fn log_file_path() -> PathBuf {
    let now = Utc::now().with_timezone(&Los_Angeles);
    let date_str = now.format("%Y-%m-%d").to_string();
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tv")
        .join("logs");
    log_dir.join(format!("tv_{date_str}.jsonl"))
}

struct PacificTime;

impl FormatTime for PacificTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Utc::now().with_timezone(&Los_Angeles);
        write!(w, "{}", now.format("%Y-%m-%dT%H:%M:%S%.3f%:z"))
    }
}

fn default_log_level() -> Level {
    if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    }
}

fn build_env_filter() -> EnvFilter {
    let default_level = default_log_level();
    EnvFilter::try_from_env("TV_LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new(default_level.as_str()))
}

struct FileAndStdout {
    file: Option<std::fs::File>,
}

impl FileAndStdout {
    fn new(log_path: &PathBuf) -> Self {
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();
        if file.is_none() {
            eprintln!("Warning: Could not open log file at {}", log_path.display());
        }
        Self { file }
    }
}

struct DualWriter {
    file: Option<std::fs::File>,
}

impl io::Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = io::Write::write(&mut io::stdout(), buf);
        if let Some(ref mut file) = self.file {
            let _ = io::Write::write(file, buf);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let _ = io::Write::flush(&mut io::stdout());
        if let Some(ref mut file) = self.file {
            let _ = io::Write::flush(file);
        }
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for FileAndStdout {
    type Writer = DualWriter;

    fn make_writer(&'a self) -> Self::Writer {
        DualWriter {
            file: self.file.as_ref().and_then(|f| f.try_clone().ok()),
        }
    }
}
