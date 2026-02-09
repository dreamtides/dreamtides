use std::collections::HashMap;

use ability_data::ability::Ability;
use ability_data::variable_value::VariableValue;
use chumsky::prelude::*;
use chumsky::span::{SimpleSpan, Span};
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use parser_v2::builder::parser_builder;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use rlf::{Phrase, Value};
use strings::strings;

/// Stack red zone size - if less than this remains, grow the stack.
/// The parser hierarchy is deep and needs significant stack during
/// construction. We use a very large red zone to ensure we catch the overflow
/// early, before the thread's stack is exhausted.
const STACK_RED_ZONE: usize = 1024 * 1024; // 1 MB

/// Stack size to grow by when needed.
const STACK_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Wrapper that ensures sufficient stack space for parser operations.
/// The deep Chumsky parser hierarchy requires significant stack during
/// construction, which can cause stack overflow in low-memory environments
/// (like Docker) when tests run in parallel.
fn with_stack<T>(f: impl FnOnce() -> T) -> T {
    stacker::maybe_grow(STACK_RED_ZONE, STACK_SIZE, f)
}

pub fn parse_ability(input: &str, vars: &str) -> Ability {
    with_stack(|| {
        let lex_result = lexer_tokenize::lex(input).unwrap();
        let bindings = VariableBindings::parse(vars).unwrap();
        let resolved =
            parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
        let parser = ability_parser::ability_parser();
        parser.parse(&resolved).into_result().unwrap()
    })
}

pub fn parse_abilities(input: &str, vars: &str) -> Vec<Ability> {
    with_stack(|| {
        let bindings = VariableBindings::parse(vars).unwrap();
        let mut abilities = Vec::new();
        for block in input.split("\n\n") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let lex_result = lexer_tokenize::lex(block).unwrap();
            let resolved =
                parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
            let parser = ability_parser::ability_parser();
            abilities.push(parser.parse(&resolved).into_result().unwrap());
        }
        abilities
    })
}

pub fn parse_spanned_ability(input: &str, vars: &str) -> SpannedAbility {
    with_stack(|| {
        let lex_result = lexer_tokenize::lex(input).unwrap();
        let bindings = VariableBindings::parse(vars).unwrap();
        let resolved =
            parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
        let parser = ability_parser::ability_parser();
        let ability = parser.parse(&resolved).into_result().unwrap();
        parser_builder::build_spanned_ability(&ability, &lex_result).unwrap()
    })
}

pub fn parse_spanned_abilities(input: &str, vars: &str) -> Vec<SpannedAbility> {
    with_stack(|| {
        let bindings = VariableBindings::parse(vars).unwrap();
        let mut abilities = Vec::new();
        for block in input.split("\n\n") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let lex_result = lexer_tokenize::lex(block).unwrap();
            let resolved =
                parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
            let parser = ability_parser::ability_parser();
            let ability = parser.parse(&resolved).into_result().unwrap();
            abilities.push(parser_builder::build_spanned_ability(&ability, &lex_result).unwrap());
        }
        abilities
    })
}

pub fn build_spanned_ability(ability: &Ability, input: &str) -> SpannedAbility {
    parser_builder::build_spanned_ability(ability, &lexer_tokenize::lex(input).unwrap()).unwrap()
}

/// Verifies that a span is valid (non-empty).
pub fn assert_valid_span(span: &SimpleSpan) {
    assert!(
        span.end() > span.start(),
        "Span should be non-empty: start={}, end={}",
        span.start(),
        span.end()
    );
}

/// Evaluates a template string with RLF variable bindings to produce rendered
/// text.
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
pub fn build_params(bindings: &VariableBindings) -> HashMap<String, Value> {
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
pub fn figment_phrase(figment: FigmentType) -> Phrase {
    match figment {
        FigmentType::Celestial => strings::celestial(),
        FigmentType::Halcyon => strings::halcyon(),
        FigmentType::Radiant => strings::radiant(),
        FigmentType::Shadow => strings::shadow(),
    }
}

/// Asserts dual-path rendered output comparison for an ability.
///
/// Path A: parse(input) -> serialize() -> serialized.text
/// Path B: eval_str(input_text, parse_bindings(vars)) -> rendered
pub fn assert_rendered_match(input_text: &str, vars: &str) {
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    let path_a = serialized.text;
    let input_bindings = VariableBindings::parse(vars).unwrap();
    let path_b = eval_str(input_text, &input_bindings);
    assert_eq!(
        path_a, path_b,
        "Rendered output mismatch:\n  Path A (serialize then render): {path_a:?}\n  Path B (render input directly): {path_b:?}\n  input: {input_text:?}\n  vars: {vars:?}"
    );
}

/// Error describing a rendered comparison failure for a TOML bulk test.
pub struct RenderedComparisonError {
    pub card_name: String,
    pub ability_text: String,
    pub variables: String,
    pub error_detail: String,
}

/// Runs a dual-path rendered comparison for a single ability from a TOML file.
///
/// Returns Ok(()) on success or Err with error details on failure.
pub fn assert_rendered_match_for_toml(
    card_name: &str,
    ability_text: &str,
    variables: &str,
) -> Result<(), RenderedComparisonError> {
    let bindings = VariableBindings::parse(variables).map_err(|e| RenderedComparisonError {
        card_name: card_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        error_detail: format!("Variable binding error: {e:?}"),
    })?;

    let lex_result = lexer_tokenize::lex(ability_text).map_err(|e| RenderedComparisonError {
        card_name: card_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        error_detail: format!("Lexer error: {e:?}"),
    })?;

    let resolved =
        parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).map_err(|e| {
            RenderedComparisonError {
                card_name: card_name.to_string(),
                ability_text: ability_text.to_string(),
                variables: variables.to_string(),
                error_detail: format!("Variable resolution error: {e}"),
            }
        })?;

    let ability = with_stack(|| {
        let parser = ability_parser::ability_parser();
        parser.parse(&resolved).into_result()
    })
    .map_err(|errors| {
        let error_msg = if errors.is_empty() {
            "Unknown parse error".to_string()
        } else {
            format!("{:?}", errors[0])
        };
        RenderedComparisonError {
            card_name: card_name.to_string(),
            ability_text: ability_text.to_string(),
            variables: variables.to_string(),
            error_detail: format!("Parse error: {error_msg}"),
        }
    })?;

    let serialized = ability_serializer::serialize_ability(&ability);
    let path_a = serialized.text;
    let path_b = eval_str(ability_text, &bindings);

    if path_a != path_b {
        return Err(RenderedComparisonError {
            card_name: card_name.to_string(),
            ability_text: ability_text.to_string(),
            variables: variables.to_string(),
            error_detail: format!(
                "Rendered output mismatch:\n  Path A (serialize then render): {path_a:?}\n  Path B (render input directly): {path_b:?}"
            ),
        });
    }

    Ok(())
}

/// Prints results for a TOML bulk rendered comparison test.
pub fn print_bulk_results(
    file_name: &str,
    success_count: usize,
    total_abilities: usize,
    errors: &[RenderedComparisonError],
) {
    println!("\n========================================");
    println!("{file_name} Rendered Comparison Results");
    println!("========================================\n");
    println!("Total abilities: {total_abilities}");
    println!("Successfully compared: {success_count}");
    println!("Failed: {}\n", errors.len());

    if errors.is_empty() {
        println!("All abilities matched rendered output!");
    } else {
        println!("Failures:\n");
        println!("{}", "=".repeat(80));

        for (i, error) in errors.iter().enumerate() {
            println!("\nFailure #{}", i + 1);
            println!("{}", "-".repeat(80));
            println!("Card: {}", error.card_name);
            println!("\nOriginal Text:");
            println!("  {}", error.ability_text.replace('\n', "\n  "));

            if !error.variables.is_empty() {
                println!("\nOriginal Variables:");
                println!("  {}", error.variables.replace('\n', "\n  "));
            }

            println!("\n{}", error.error_detail);
            println!("{}", "-".repeat(80));
        }
    }
    println!("\n========================================\n");
}
