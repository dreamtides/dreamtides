use serde::Deserialize;

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
