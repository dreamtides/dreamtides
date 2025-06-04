use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DebugPromptData {
    pub is_active: bool,
    pub player: String,
    pub prompt_kind: String,
    pub choices: Vec<String>,
    pub configuration: String,
}
