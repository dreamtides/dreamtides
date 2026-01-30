use battle_state::prompt_types::prompt_data::PromptChoiceLabel;
use fluent::{FluentArgs, fluent_args};

use crate::core::response_builder::ResponseBuilder;

pub fn choice_label(builder: &ResponseBuilder, label: PromptChoiceLabel) -> String {
    match label {
        PromptChoiceLabel::String(string_id) => {
            builder.string_with_args(string_id, FluentArgs::new())
        }
        PromptChoiceLabel::StringWithEnergy(string_id, energy) => {
            builder.string_with_args(string_id, fluent_args!("e" => energy))
        }
    }
}
