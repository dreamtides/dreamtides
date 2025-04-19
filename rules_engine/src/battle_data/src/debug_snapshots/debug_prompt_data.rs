use crate::prompt_types::prompt_data::PromptData;

pub struct DebugPromptData {
    pub player: String,
    pub prompt: String,
    pub options: String,
    pub context: String,
}

impl DebugPromptData {
    pub fn new(prompt_data: PromptData) -> Self {
        Self {
            player: format!("{:?}", prompt_data.player),
            prompt: format!("{:?}", prompt_data.prompt),
            options: format!("{:?}", prompt_data.options),
            context: format!("{:?}", prompt_data.context),
        }
    }
}
