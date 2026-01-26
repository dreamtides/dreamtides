use crate::logging::log_aggregator::{self, FrontendLogMessage};

/// Tauri command to receive a log message from the frontend.
#[tauri::command]
pub fn log_message(message: FrontendLogMessage) {
    log_aggregator::ingest(&message);
}
