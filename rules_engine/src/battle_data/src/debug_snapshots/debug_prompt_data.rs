use serde::Serialize;
use strum::IntoDiscriminant;

use crate::prompt_types::prompt_data::{Prompt, PromptData};

#[derive(Debug, Clone, Serialize)]
pub struct DebugPromptData {
    pub player: String,
    pub prompt_kind: String,
    pub choices: Vec<String>,
    pub configuration: String,
    pub context: String,
}

impl DebugPromptData {
    pub fn new(prompt_data: PromptData) -> Self {
        Self {
            player: format!("{:?}", prompt_data.player),
            prompt_kind: format!("{:?}", prompt_data.prompt.discriminant()),
            choices: format_prompt_choices(&prompt_data.prompt),
            configuration: format!("{:?}", prompt_data.configuration),
            context: format!("{:?}", prompt_data.context),
        }
    }
}

fn format_prompt_choices(prompt: &Prompt) -> Vec<String> {
    match prompt {
        Prompt::ChooseCharacter { valid } => valid.iter().map(|id| format!("{:?}", id)).collect(),
        Prompt::ChooseStackCard { valid } => valid.iter().map(|id| format!("{:?}", id)).collect(),
        Prompt::Choose { choices } => {
            choices.iter().map(|choice| format!("{:?}", choice)).collect()
        }
        Prompt::ChooseNumber { minimum, current, maximum } => {
            vec![format!("{}", minimum), format!("{}", current), format!("{}", maximum)]
        }
    }
}
