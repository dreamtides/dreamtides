use std::io;
use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use chrono_tz::America::Los_Angeles;
use tracing::Level;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};

/// Initializes the tracing subscriber with dual layers: JSON file and compact stdout.
pub fn initialize() {
    let log_path = log_file_path();
    let file_writer = FileWriter::new(&log_path);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .flatten_event(true)
        .with_timer(PacificTime)
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .with_writer(file_writer);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_timer(PacificTime)
        .with_target(false)
        .with_writer(std::io::stdout);

    Registry::default()
        .with(build_env_filter())
        .with(file_layer)
        .with(build_env_filter())
        .with(stdout_layer)
        .init();

    tracing::info!(
        component = "tv.logging",
        log_file = %log_path.display(),
        "Logging initialized"
    );

    cleanup_old_logs();
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
    EnvFilter::try_from_env("TV_LOG_LEVEL").unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{},hyper=warn,reqwest=warn,tao=warn,wry=warn,tauri=warn",
            default_log_level().as_str().to_lowercase()
        ))
    })
}

fn cleanup_old_logs() {
    let log_dir = log_file_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let retention_days: i64 = std::env::var("TV_LOG_RETENTION_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    let today = Utc::now().with_timezone(&Los_Angeles).date_naive();

    let entries = match std::fs::read_dir(&log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };

        let Some(date_str) = name.strip_prefix("tv_").and_then(|s| s.strip_suffix(".jsonl")) else {
            continue;
        };

        let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            continue;
        };

        if (today - file_date).num_days() > retention_days {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                tracing::warn!(
                    component = "tv.logging",
                    file = %entry.path().display(),
                    error = %e,
                    "Failed to delete old log file"
                );
            }
        }
    }
}

struct FileWriter {
    file: Option<std::fs::File>,
}

impl FileWriter {
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

struct FileOnlyWriter {
    file: Option<std::fs::File>,
}

impl io::Write for FileOnlyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref mut file) = self.file {
            io::Write::write(file, buf)
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            io::Write::flush(file)
        } else {
            Ok(())
        }
    }
}

impl<'a> MakeWriter<'a> for FileWriter {
    type Writer = FileOnlyWriter;

    fn make_writer(&'a self) -> Self::Writer {
        FileOnlyWriter {
            file: self.file.as_ref().and_then(|f| f.try_clone().ok()),
        }
    }
}
