use battle_state::prompt_types::prompt_data::PromptChoiceLabel;
use rlf::Value;
use strings::strings;

/// Renders a [PromptChoiceLabel] to its display string.
pub fn choice_label(label: PromptChoiceLabel) -> String {
    strings::register_source_phrases();
    match label {
        PromptChoiceLabel::String(id) => {
            id.resolve_global().expect("phrase should exist").to_string()
        }
        PromptChoiceLabel::StringWithEnergy(id, energy) => {
            id.call_global(&[Value::from(energy.0)]).expect("phrase should exist").to_string()
        }
    }
}
