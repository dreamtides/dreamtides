//! Golden file test for rendered card text output.
//!
//! Generates rendered text for every card ability in cards.toml and
//! dreamwell.toml and compares it against a stored baseline file.

use std::collections::HashMap;

use ability_data::variable_value::VariableValue;
use chumsky::Parser;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use rlf::{Phrase, Value};
use serde::Deserialize;
use strings::strings;

#[derive(Debug, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DreamwellFile {
    dreamwell: Vec<Dreamwell>,
}

#[derive(Debug, Deserialize)]
struct Dreamwell {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

/// Evaluates a template string with RLF variable bindings.
fn eval_str(template: &str, bindings: &VariableBindings) -> String {
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

/// Generates a single golden file entry for one ability.
fn generate_entry(
    name: &str,
    ability_index: usize,
    ability_text: &str,
    variables: &str,
) -> Result<String, String> {
    let bindings = VariableBindings::parse(variables)
        .map_err(|e| format!("{name}|{ability_index}|ERROR: variable parse: {e:?}"))?;

    let lex_result = lexer_tokenize::lex(ability_text)
        .map_err(|e| format!("{name}|{ability_index}|ERROR: lex: {e:?}"))?;

    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .map_err(|e| format!("{name}|{ability_index}|ERROR: resolve: {e}"))?;

    let ability = {
        let parser = ability_parser::ability_parser();
        parser
            .parse(&resolved)
            .into_result()
            .map_err(|e| format!("{name}|{ability_index}|ERROR: parse: {e:?}"))?
    };

    let serialized = ability_serializer::serialize_ability(&ability);
    let rendered = eval_str(&serialized.text, &serialized.variables);

    Ok(format!("{name}|{ability_index}|{rendered}"))
}

/// Generates rendered output lines for all abilities in a card-style TOML file.
fn generate_card_entries(cards_toml: &str, entries: &mut Vec<String>, errors: &mut Vec<String>) {
    let cards_file: CardsFile = toml::from_str(cards_toml).expect("Failed to parse cards TOML");

    for card in &cards_file.cards {
        let Some(rules_text) = &card.rules_text else {
            continue;
        };

        let variables = card.variables.as_deref().unwrap_or("");

        for (ability_index, ability_block) in rules_text.split("\n\n").enumerate() {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            match generate_entry(&card.name, ability_index, ability_block, variables) {
                Ok(entry) => entries.push(entry),
                Err(e) => errors.push(e),
            }
        }
    }
}

/// Generates rendered output lines for all abilities in a dreamwell TOML file.
fn generate_dreamwell_entries(
    dreamwell_toml: &str,
    entries: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let dreamwell_file: DreamwellFile =
        toml::from_str(dreamwell_toml).expect("Failed to parse dreamwell TOML");

    for dreamwell in &dreamwell_file.dreamwell {
        let Some(rules_text) = &dreamwell.rules_text else {
            continue;
        };

        let variables = dreamwell.variables.as_deref().unwrap_or("");

        for (ability_index, ability_block) in rules_text.split("\n\n").enumerate() {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            match generate_entry(&dreamwell.name, ability_index, ability_block, variables) {
                Ok(entry) => entries.push(entry),
                Err(e) => errors.push(e),
            }
        }
    }
}

/// Generates all golden file content, sorted by entry.
fn generate_golden_content() -> String {
    let cards_toml =
        std::fs::read_to_string("../../tabula/cards.toml").expect("Failed to read cards.toml");
    let dreamwell_toml = std::fs::read_to_string("../../tabula/dreamwell.toml")
        .expect("Failed to read dreamwell.toml");

    let mut entries = Vec::new();
    let mut errors = Vec::new();

    generate_card_entries(&cards_toml, &mut entries, &mut errors);
    generate_dreamwell_entries(&dreamwell_toml, &mut entries, &mut errors);

    if !errors.is_empty() {
        panic!("Failed to generate {} golden file entries:\n{}", errors.len(), errors.join("\n"));
    }

    entries.sort();
    let mut content = entries.join("\n");
    content.push('\n');
    content
}

#[test]
fn test_golden_rendered_output() {
    let golden_path =
        std::path::PathBuf::from("tests/round_trip_tests/fixtures/golden_rendered_output.txt");

    let generated = generate_golden_content();

    if !golden_path.exists() {
        std::fs::write(&golden_path, &generated).unwrap_or_else(|e| {
            panic!("Failed to write golden file at {}: {e}", golden_path.display())
        });
        println!(
            "Generated golden file at {} with {} entries",
            golden_path.display(),
            generated.lines().count()
        );
        return;
    }

    let stored = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("Failed to read golden file at {}: {e}", golden_path.display()));

    if generated != stored {
        let generated_lines: Vec<&str> = generated.lines().collect();
        let stored_lines: Vec<&str> = stored.lines().collect();

        let mut diffs = Vec::new();
        let max_lines = generated_lines.len().max(stored_lines.len());
        for i in 0..max_lines {
            let gen_line = generated_lines.get(i).copied().unwrap_or("<missing>");
            let stored_line = stored_lines.get(i).copied().unwrap_or("<missing>");
            if gen_line != stored_line {
                diffs.push(format!("  line {}: expected {stored_line:?}, got {gen_line:?}", i + 1));
            }
        }

        panic!(
            "Golden file mismatch ({} differences):\n{}\n\n\
             To update the golden file, delete it and re-run this test.",
            diffs.len(),
            diffs.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
        );
    }
}
