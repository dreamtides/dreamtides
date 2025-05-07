use serde::Serialize;
use strum::IntoDiscriminant;

use crate::prompt_types::prompt_data::{PromptData, PromptType};

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
            prompt_kind: format!("{:?}", prompt_data.prompt_type.discriminant()),
            choices: format_prompt_choices(&prompt_data.prompt_type),
            configuration: format!("{:?}", prompt_data.configuration),
            context: format!("{:?}", prompt_data.context),
        }
    }
}

fn format_prompt_choices(prompt: &PromptType) -> Vec<String> {
    match prompt {
        PromptType::ChooseCharacter { valid } => {
            valid.iter().map(|id| format!("{:?}", id)).collect()
        }
        PromptType::ChooseStackCard { valid } => {
            valid.iter().map(|id| format!("{:?}", id)).collect()
        }
        PromptType::Choose { choices } => {
            choices.iter().map(|choice| format!("{:?}", choice)).collect()
        }
        PromptType::ChooseEnergyValue { minimum, current, maximum } => {
            vec![
                format!("min {}", minimum),
                format!("current {}", current),
                format!("max {}", maximum),
            ]
        }
    }
}
