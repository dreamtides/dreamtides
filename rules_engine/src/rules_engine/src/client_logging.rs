use display_data::client_log_request::{ClientLogRequest, LogEntry, LogSpanName, LogType};
use tracing::{debug, error, info, warn};

/// Logs events from the client via the 'tracing' crate.
pub fn log_client_events(request: ClientLogRequest) {
    log_entry(&request.entry);
}

fn log_entry(entry: &LogEntry) {
    match entry {
        LogEntry::Event { log_type, message } => match log_type {
            LogType::Debug => {
                debug!("{}", message);
            }
            LogType::Info => {
                info!("{}", message);
            }
            LogType::Warning => {
                warn!("{}", message);
            }
            LogType::Error => {
                error!("{}", message);
            }
        },
        LogEntry::EventSpan { name, entries } => {
            let span = match name {
                LogSpanName::Untagged => {
                    tracing::span!(tracing::Level::DEBUG, "client_span")
                }
                LogSpanName::Connect => {
                    tracing::span!(tracing::Level::DEBUG, "connect")
                }
                LogSpanName::PerformAction => {
                    tracing::span!(tracing::Level::DEBUG, "perform_action")
                }
                LogSpanName::Poll => {
                    tracing::span!(tracing::Level::DEBUG, "poll")
                }
                LogSpanName::ApplyCommands => {
                    tracing::span!(tracing::Level::DEBUG, "apply_commands")
                }
                LogSpanName::ApplyCommandGroup => {
                    tracing::span!(tracing::Level::DEBUG, "apply_command_group")
                }
                LogSpanName::UpdateBattleLayout => {
                    tracing::span!(tracing::Level::DEBUG, "update_battle_layout")
                }
            };

            let _guard = span.enter();

            for nested_entry in entries {
                log_entry(nested_entry);
            }
        }
    }
}
