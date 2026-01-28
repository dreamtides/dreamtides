use serde::Deserialize;

use super::json_logger;

/// A log message received from the frontend.
#[derive(Debug, Deserialize)]
pub struct FrontendLogMessage {
    pub ts: String,
    pub level: String,
    pub component: String,
    pub msg: String,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

/// A performance log entry received from the frontend.
/// These are written to a dedicated performance log file for focused analysis.
#[derive(Debug, Deserialize)]
pub struct FrontendPerfLogEntry {
    /// Timestamp when the operation started (ISO 8601 format in Pacific time).
    pub ts: String,
    /// Component where the operation occurred.
    pub component: String,
    /// Name of the operation being measured.
    pub operation: String,
    /// Duration of the operation in milliseconds.
    pub duration_ms: f64,
    /// Optional additional context about the operation.
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}

/// Ingests a frontend log message into the backend tracing system.
pub fn ingest(message: &FrontendLogMessage) {
    let context_str = message
        .context
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_default();

    match message.level.as_str() {
        "ERROR" => {
            tracing::error!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                "{}", message.msg
            );
        }
        "WARN" => {
            tracing::warn!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                "{}", message.msg
            );
        }
        "INFO" => {
            tracing::info!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                "{}", message.msg
            );
        }
        "DEBUG" => {
            tracing::debug!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                "{}", message.msg
            );
        }
        "TRACE" => {
            tracing::trace!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                "{}", message.msg
            );
        }
        _ => {
            tracing::info!(
                component = %message.component,
                frontend_ts = %message.ts,
                context = %context_str,
                unknown_level = %message.level,
                "{}", message.msg
            );
        }
    }
}

/// Ingests a frontend performance log entry into the dedicated performance log file.
pub fn ingest_perf(entry: &FrontendPerfLogEntry) {
    // Build JSON object for performance log
    let mut log_entry = serde_json::json!({
        "ts": entry.ts,
        "component": entry.component,
        "operation": entry.operation,
        "duration_ms": entry.duration_ms,
    });

    if let Some(ref ctx) = entry.context {
        log_entry["context"] = ctx.clone();
    }

    json_logger::write_perf_log(&log_entry);
}
