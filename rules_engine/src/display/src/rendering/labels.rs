use battle_state::prompt_types::prompt_data::PromptChoiceLabel;
use ui_components::icon;

pub fn choice_label(label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::Decline => "Decline".to_string(),
        PromptChoiceLabel::PayEnergy(energy) => format!("Spend {}{}", energy, icon::ENERGY),
    }
}
