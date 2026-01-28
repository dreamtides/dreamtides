use crate::logging::log_aggregator::{self, FrontendLogMessage, FrontendPerfLogEntry};

/// Tauri command to receive a log message from the frontend.
#[tauri::command]
pub fn log_message(message: FrontendLogMessage) {
    log_aggregator::ingest(&message);
}

/// Tauri command to receive a performance log entry from the frontend.
/// Performance entries are written to a dedicated log file for focused analysis.
#[tauri::command]
pub fn log_perf(entry: FrontendPerfLogEntry) {
    log_aggregator::ingest_perf(&entry);
}
