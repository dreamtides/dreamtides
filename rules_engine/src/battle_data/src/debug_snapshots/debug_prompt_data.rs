use crate::prompts::prompt_data::PromptData;

pub struct DebugPromptData {
    pub player: String,
    pub prompt: String,
    pub optional: String,
    pub context: String,
}

impl DebugPromptData {
    pub fn new(prompt_data: PromptData) -> Self {
        Self {
            player: format!("{:?}", prompt_data.player),
            prompt: format!("{:?}", prompt_data.prompt),
            optional: format!("{}", prompt_data.optional),
            context: format!("{:?}", prompt_data.context),
        }
    }
}
