use std::collections::HashMap;

use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use rlf::{Phrase, Value};
use strings::strings;

/// Ensures source phrases are registered in the global RLF locale.
fn ensure_phrases_registered() {
    strings::register_source_phrases();
}

/// Parsed variable key-value pairs that can be converted to RLF params.
#[derive(Debug, Clone)]
pub struct ParsedVariables {
    pairs: Vec<(String, String)>,
}

impl ParsedVariables {
    /// Converts the parsed variables into RLF parameter values.
    ///
    /// Subtypes and figment types are resolved to RLF Phrase values
    /// (with article tags and plural variants) so that `:from` modifiers
    /// work correctly.
    pub fn to_rlf_params(&self) -> HashMap<String, Value> {
        ensure_phrases_registered();
        let mut params = HashMap::new();
        for (key, value) in &self.pairs {
            let rlf_value = if let Ok(n) = value.parse::<i64>() {
                Value::Number(n)
            } else if let Some(subtype) = CardSubtype::from_variable(value) {
                Value::Phrase(subtype_phrase(subtype))
            } else if let Some(figment) = FigmentType::from_variable(value) {
                Value::Phrase(figment_phrase(figment))
            } else {
                rlf::with_locale(|locale| match locale.get_phrase(value) {
                    Ok(phrase) => Value::Phrase(phrase),
                    Err(_) => Value::String(value.clone()),
                })
            };
            params.insert(key.clone(), rlf_value);
        }
        params
    }
}

fn subtype_phrase(subtype: CardSubtype) -> Phrase {
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

fn figment_phrase(figment: FigmentType) -> Phrase {
    match figment {
        FigmentType::Celestial => strings::celestial(),
        FigmentType::Halcyon => strings::halcyon(),
        FigmentType::Radiant => strings::radiant(),
        FigmentType::Shadow => strings::shadow(),
    }
}

/// Parses a variables string into ParsedVariables.
pub fn parse_variables(variables_text: &str) -> Result<ParsedVariables, String> {
    let mut pairs = Vec::new();

    for line in variables_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(format!("Invalid variable definition: '{line}'"));
        };

        let key = key.trim();
        let value = value.trim();

        if key.is_empty() || value.is_empty() {
            return Err(format!("Invalid variable definition: '{line}'"));
        }

        pairs.push((key.to_string(), value.to_string()));
    }

    Ok(ParsedVariables { pairs })
}

/// Formats an expression using the global RLF locale and provided parameters.
pub fn format_expression(
    expression: &str,
    params: HashMap<String, Value>,
) -> Result<String, String> {
    ensure_phrases_registered();

    tracing::debug!(
        component = "derived.rules_preview",
        expression_len = expression.len(),
        param_count = params.len(),
        "Formatting RLF expression"
    );

    rlf::with_locale(|locale| {
        locale
            .eval_str(expression, params)
            .map(|phrase| phrase.to_string())
            .map_err(|e| format!("RLF eval error: {e}"))
    })
}
