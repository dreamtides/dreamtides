use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientLogRequest {
    pub entry: LogEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LogEntry {
    Event { log_type: LogType, message: String, arguments: BTreeMap<String, String> },
    EventSpan { name: String, entries: Vec<LogEntry> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LogType {
    Warning,
    Error,
    Info,
    Debug,
}
