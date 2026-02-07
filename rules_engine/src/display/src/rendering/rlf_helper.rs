use std::collections::HashMap;

use ability_data::variable_value::VariableValue;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use parser_v2::variables::parser_bindings::VariableBindings;
use rlf::Value;
use strings::strings;

/// Evaluates a template string with RLF variable bindings.
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

/// Converts [VariableBindings] to RLF parameters.
fn build_params(bindings: &VariableBindings) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    for (name, value) in bindings.iter() {
        let rlf_value = match value {
            VariableValue::Integer(n) => Value::Number(*n as i64),
            VariableValue::Subtype(subtype) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale
                        .get_phrase(subtype_phrase_name(*subtype))
                        .expect("subtype phrase should exist"),
                )
            }),
            VariableValue::Figment(figment) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale
                        .get_phrase(figment_phrase_name(*figment))
                        .expect("figment phrase should exist"),
                )
            }),
        };
        params.insert(name.replace('-', "_"), rlf_value);
    }
    params
}

/// Returns the RLF phrase name for a [CardSubtype].
fn subtype_phrase_name(subtype: CardSubtype) -> &'static str {
    match subtype {
        CardSubtype::Agent => "agent",
        CardSubtype::Ancient => "ancient",
        CardSubtype::Avatar => "avatar",
        CardSubtype::Child => "child",
        CardSubtype::Detective => "detective",
        CardSubtype::Enigma => "enigma",
        CardSubtype::Explorer => "explorer",
        CardSubtype::Guide => "guide",
        CardSubtype::Hacker => "hacker",
        CardSubtype::Mage => "mage",
        CardSubtype::Monster => "monster",
        CardSubtype::Musician => "musician",
        CardSubtype::Outsider => "outsider",
        CardSubtype::Renegade => "renegade",
        CardSubtype::Robot => "robot",
        CardSubtype::SpiritAnimal => "spirit_animal",
        CardSubtype::Super => "super_",
        CardSubtype::Survivor => "survivor",
        CardSubtype::Synth => "synth",
        CardSubtype::Tinkerer => "tinkerer",
        CardSubtype::Trooper => "trooper",
        CardSubtype::Visionary => "visionary",
        CardSubtype::Visitor => "visitor",
        CardSubtype::Warrior => "warrior",
    }
}

/// Returns the RLF phrase name for a [FigmentType].
fn figment_phrase_name(figment: FigmentType) -> &'static str {
    match figment {
        FigmentType::Celestial => "celestial",
        FigmentType::Halcyon => "halcyon",
        FigmentType::Radiant => "radiant",
        FigmentType::Shadow => "shadow",
    }
}
