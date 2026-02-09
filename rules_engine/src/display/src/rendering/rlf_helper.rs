use std::collections::HashMap;

use ability_data::variable_value::VariableValue;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use parser_v2::variables::parser_bindings::VariableBindings;
use rlf::{Phrase, Value};
use strings::strings;

/// Evaluates a template string with RLF variable bindings. Used by the test
/// oracle for dual-path rendered comparison.
pub fn eval_str(template: &str, bindings: &VariableBindings) -> String {
    strings::register_source_phrases();
    let params = build_params(bindings);
    rlf::with_locale(|locale| {
        locale
            .eval_str(template, params)
            .unwrap_or_else(|e| panic!("Error evaluating template {template:?}: {e}"))
            .to_string()
    })
}

/// Returns the RLF phrase for a [CardSubtype].
pub fn subtype_phrase(subtype: CardSubtype) -> Phrase {
    match subtype {
        CardSubtype::Agent => strings::agent(),
        CardSubtype::Ancient => strings::ancient(),
        CardSubtype::Avatar => strings::avatar(),
        CardSubtype::Child => strings::child(),
        CardSubtype::Detective => strings::detective(),
        CardSubtype::Enigma => strings::enigma(),
        CardSubtype::Explorer => strings::explorer(),
        CardSubtype::Guide => strings::guide(),
        CardSubtype::Hacker => strings::hacker(),
        CardSubtype::Mage => strings::mage(),
        CardSubtype::Monster => strings::monster(),
        CardSubtype::Musician => strings::musician(),
        CardSubtype::Outsider => strings::outsider(),
        CardSubtype::Renegade => strings::renegade(),
        CardSubtype::Robot => strings::robot(),
        CardSubtype::SpiritAnimal => strings::spirit_animal(),
        CardSubtype::Super => strings::super_(),
        CardSubtype::Survivor => strings::survivor(),
        CardSubtype::Synth => strings::synth(),
        CardSubtype::Tinkerer => strings::tinkerer(),
        CardSubtype::Trooper => strings::trooper(),
        CardSubtype::Visionary => strings::visionary(),
        CardSubtype::Visitor => strings::visitor(),
        CardSubtype::Warrior => strings::warrior(),
    }
}

/// Converts [VariableBindings] to RLF parameters.
fn build_params(bindings: &VariableBindings) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    for (name, value) in bindings.iter() {
        let rlf_value = match value {
            VariableValue::Integer(n) => Value::Number(*n as i64),
            VariableValue::Subtype(subtype) => Value::Phrase(subtype_phrase(*subtype)),
            VariableValue::Figment(figment) => Value::Phrase(figment_phrase(*figment)),
        };
        params.insert(name.clone(), rlf_value);
    }
    params
}

/// Returns the RLF phrase for a [FigmentType].
fn figment_phrase(figment: FigmentType) -> Phrase {
    match figment {
        FigmentType::Celestial => strings::celestial(),
        FigmentType::Halcyon => strings::halcyon(),
        FigmentType::Radiant => strings::radiant(),
        FigmentType::Shadow => strings::shadow(),
    }
}
