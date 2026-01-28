use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::{NaiveDate, Utc};
use chrono_tz::America::Los_Angeles;
use parking_lot::Mutex;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry};

/// Global performance log file writer.
static PERF_LOG_WRITER: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

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
        .with_writer(file_writer)
        .with_filter(build_env_filter());

    let stdout_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_timer(PacificTime)
        .with_target(false)
        .with_writer(std::io::stdout)
        .with_filter(LevelFilter::ERROR);

    Registry::default()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    // Initialize performance log file
    let perf_path = perf_log_file_path();
    initialize_perf_log(&perf_path);

    tracing::info!(
        component = "tv.logging",
        log_file = %log_path.display(),
        perf_log_file = %perf_path.display(),
        "Logging initialized"
    );

    cleanup_old_logs();
}

/// Returns the path for today's main log file.
pub fn log_file_path() -> PathBuf {
    let now = Utc::now().with_timezone(&Los_Angeles);
    let date_str = now.format("%Y-%m-%d").to_string();
    log_dir().join(format!("tv_{date_str}.jsonl"))
}

/// Returns the path for today's performance log file.
pub fn perf_log_file_path() -> PathBuf {
    let now = Utc::now().with_timezone(&Los_Angeles);
    let date_str = now.format("%Y-%m-%d").to_string();
    log_dir().join(format!("tv_perf_{date_str}.jsonl"))
}

/// Returns the log directory path.
fn log_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tv")
        .join("logs")
}

/// Initializes the performance log file writer.
fn initialize_perf_log(perf_path: &PathBuf) {
    if let Some(parent) = perf_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(perf_path)
        .ok();
    if file.is_none() {
        eprintln!(
            "Warning: Could not open performance log file at {}",
            perf_path.display()
        );
    }
    let _ = PERF_LOG_WRITER.set(Mutex::new(file));
}

/// Writes a performance log entry to the dedicated performance log file.
/// This is separate from the main tracing log to allow focused analysis
/// of frontend performance metrics without noise from other log entries.
pub fn write_perf_log(entry: &serde_json::Value) {
    let writer = PERF_LOG_WRITER.get_or_init(|| Mutex::new(None));
    let mut guard = writer.lock();
    if let Some(ref mut file) = *guard {
        let _ = serde_json::to_writer(&mut *file, entry);
        let _ = io::Write::write_all(file, b"\n");
        let _ = io::Write::flush(file);
    }
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

        // Handle both main log files (tv_YYYY-MM-DD.jsonl) and perf log files (tv_perf_YYYY-MM-DD.jsonl)
        let date_str = if let Some(s) = name.strip_prefix("tv_perf_").and_then(|s| s.strip_suffix(".jsonl")) {
            s
        } else if let Some(s) = name.strip_prefix("tv_").and_then(|s| s.strip_suffix(".jsonl")) {
            s
        } else {
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
