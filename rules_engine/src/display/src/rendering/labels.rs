use battle_state::prompt_types::prompt_data::PromptChoiceLabel;

pub fn choice_label(label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::String(_) => "Decline".to_string(),
        PromptChoiceLabel::StringWithEnergy(_, energy) => {
            format!("Spend {energy}")
        }
    }
}
