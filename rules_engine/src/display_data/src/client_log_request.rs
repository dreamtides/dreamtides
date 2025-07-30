use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClientLogRequest {
    pub entry: LogEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum LogEntry {
    Event { log_type: LogType, message: String },
    EventSpan { name: LogSpanName, entries: Vec<LogEntry> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum LogType {
    Warning,
    Error,
    Info,
    Debug,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum LogSpanName {
    Untagged,
    Connect,
    PerformAction,
    Poll,
    ApplyCommands,
    ApplyCommandGroup,
    UpdateBattleLayout,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClientLogResponse {
    pub success: bool,
}
