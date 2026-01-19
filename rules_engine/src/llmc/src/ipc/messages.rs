use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HookEvent {
    SessionStart { worker: String, session_id: String, timestamp: u64 },
    SessionEnd { worker: String, reason: String, timestamp: u64 },
    Stop { worker: String, session_id: String, timestamp: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMessage {
    pub version: u8,
    pub id: Uuid,
    pub event: HookEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeHookInput {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<String>,
    pub reason: Option<String>,
}

impl HookMessage {
    pub fn new(event: HookEvent) -> Self {
        Self { version: 1, id: Uuid::new_v4(), event }
    }
}

impl HookResponse {
    pub fn success() -> Self {
        Self { success: true, error: None }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self { success: false, error: Some(msg.into()) }
    }
}
