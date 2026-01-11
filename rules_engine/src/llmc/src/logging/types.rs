#![allow(dead_code)]

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitState {
    pub branch: String,
    pub head: String,
    pub clean: bool,
}

#[derive(Debug, Serialize)]
pub struct GitOperationLog {
    pub operation_type: String,
    pub repo_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<GitState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<GitState>,
    pub args: serde_json::Value,
    pub result: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conflicts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TmuxSendLog {
    pub send_type: String,
    pub message_size_bytes: usize,
    pub message_truncated: String,
    pub debounce_ms: u64,
    pub retry_count: u32,
    pub partial_send_detected: bool,
    pub partial_status: String,
}

#[derive(Debug, Serialize)]
pub struct StateTransitionLog {
    pub from_status: String,
    pub to_status: String,
    pub transition_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
