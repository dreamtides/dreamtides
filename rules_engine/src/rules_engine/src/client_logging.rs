use display_data::client_log_request::{ClientLogRequest, LogEntry, LogType};
use tracing::{debug, error, info, warn};

/// Logs events from the client via the 'tracing' crate.
pub fn log_client_events(request: ClientLogRequest) {
    log_entry(&request.entry);
}

fn log_entry(entry: &LogEntry) {
    match entry {
        LogEntry::Event { log_type, message, arguments } => {
            let fields =
                arguments.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>();

            match log_type {
                LogType::Debug => {
                    debug!(?fields, "{}", message);
                }
                LogType::Info => {
                    info!(?fields, "{}", message);
                }
                LogType::Warning => {
                    warn!(?fields, "{}", message);
                }
                LogType::Error => {
                    error!(?fields, "{}", message);
                }
            }
        }
        LogEntry::EventSpan { name, entries } => {
            let span = tracing::span!(tracing::Level::INFO, "client_span", name);
            let _guard = span.enter();

            for nested_entry in entries {
                log_entry(nested_entry);
            }
        }
    }
}
