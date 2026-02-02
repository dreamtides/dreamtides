//! Round-trip tests for all dreamwells in dreamwell.toml.
//!
//! Verifies that parsing and serializing each dreamwell's rules text produces
//! the original text and variable bindings.

use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use serde::Deserialize;

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

struct ResolvedAbility {
    dreamwell_name: String,
    ability_text: String,
    variables: String,
    bindings: VariableBindings,
    resolved_tokens: Vec<(parser_substitutions::ResolvedToken, chumsky::span::SimpleSpan)>,
}

struct RoundTripError {
    dreamwell_name: String,
    ability_text: String,
    variables: String,
    serialized_text: String,
    serialized_variables: String,
    error_type: RoundTripErrorType,
}

enum RoundTripErrorType {
    TextMismatch,
    VariableMismatch,
    ParseError(String),
}

#[test]
fn test_all_dreamwell_toml_round_trip() {
    let dreamwell_toml = std::fs::read_to_string("../../tabula/dreamwell.toml")
        .expect("Failed to read dreamwell.toml");
    let dreamwell_file: DreamwellFile =
        toml::from_str(&dreamwell_toml).expect("Failed to parse dreamwell.toml");

    let mut resolved_abilities = Vec::new();
    let mut resolution_errors = Vec::new();

    for dreamwell in &dreamwell_file.dreamwell {
        let Some(rules_text) = &dreamwell.rules_text else {
            continue;
        };

        let variables = dreamwell.variables.as_deref().unwrap_or("");

        for ability_block in rules_text.split("\n\n") {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            match resolve_ability(&dreamwell.name, ability_block, variables) {
                Ok(resolved) => resolved_abilities.push(resolved),
                Err(error) => resolution_errors.push(error),
            }
        }
    }

    let parser = ability_parser::ability_parser();

    let mut round_trip_errors = Vec::new();
    let mut success_count = 0;

    for resolved in &resolved_abilities {
        match parser.parse(&resolved.resolved_tokens).into_result() {
            Ok(ability) => {
                let serialized = ability_serializer::serialize_ability(&ability);

                if serialized.text != resolved.ability_text {
                    round_trip_errors.push(RoundTripError {
                        dreamwell_name: resolved.dreamwell_name.clone(),
                        ability_text: resolved.ability_text.clone(),
                        variables: resolved.variables.clone(),
                        serialized_text: serialized.text,
                        serialized_variables: format!("{:?}", serialized.variables),
                        error_type: RoundTripErrorType::TextMismatch,
                    });
                } else if !variables_match(&serialized.variables, &resolved.bindings) {
                    // Check that each serialized variable matches the original binding.
                    // We don't require exact equality since the card may have variables
                    // shared across multiple abilities.
                    round_trip_errors.push(RoundTripError {
                        dreamwell_name: resolved.dreamwell_name.clone(),
                        ability_text: resolved.ability_text.clone(),
                        variables: resolved.variables.clone(),
                        serialized_text: serialized.text,
                        serialized_variables: format!("{:?}", serialized.variables),
                        error_type: RoundTripErrorType::VariableMismatch,
                    });
                } else {
                    success_count += 1;
                }
            }
            Err(errors) => {
                let error_msg = if errors.is_empty() {
                    "Unknown parse error".to_string()
                } else {
                    format!("{:?}", errors[0])
                };
                round_trip_errors.push(RoundTripError {
                    dreamwell_name: resolved.dreamwell_name.clone(),
                    ability_text: resolved.ability_text.clone(),
                    variables: resolved.variables.clone(),
                    serialized_text: String::new(),
                    serialized_variables: String::new(),
                    error_type: RoundTripErrorType::ParseError(error_msg),
                });
            }
        }
    }

    let all_errors: Vec<RoundTripError> =
        resolution_errors.into_iter().chain(round_trip_errors).collect();

    let total_abilities = resolved_abilities.len() + all_errors.len();
    print_results("dreamwell.toml", success_count, total_abilities, &all_errors);

    if !all_errors.is_empty() {
        panic!("\n{} abilities failed round-trip (see details above)", all_errors.len());
    }
}

fn resolve_ability(
    dreamwell_name: &str,
    ability_text: &str,
    variables: &str,
) -> Result<ResolvedAbility, RoundTripError> {
    let bindings = VariableBindings::parse(variables).map_err(|e| RoundTripError {
        dreamwell_name: dreamwell_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        serialized_text: String::new(),
        serialized_variables: String::new(),
        error_type: RoundTripErrorType::ParseError(format!("Variable binding error: {:?}", e)),
    })?;

    let lex_result = lexer_tokenize::lex(ability_text).map_err(|e| RoundTripError {
        dreamwell_name: dreamwell_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        serialized_text: String::new(),
        serialized_variables: String::new(),
        error_type: RoundTripErrorType::ParseError(format!("Lexer error: {:?}", e)),
    })?;

    let resolved_tokens = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .map_err(|e| RoundTripError {
            dreamwell_name: dreamwell_name.to_string(),
            ability_text: ability_text.to_string(),
            variables: variables.to_string(),
            serialized_text: String::new(),
            serialized_variables: String::new(),
            error_type: RoundTripErrorType::ParseError(format!("Variable resolution error: {}", e)),
        })?;

    Ok(ResolvedAbility {
        dreamwell_name: dreamwell_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        bindings,
        resolved_tokens,
    })
}

/// Checks that all serialized variables match the original bindings.
///
/// Returns true if every variable in `serialized` has the same value in
/// `original`. This allows the original to contain extra variables (for cards
/// with multiple abilities sharing the same variables).
fn variables_match(serialized: &VariableBindings, original: &VariableBindings) -> bool {
    for (key, value) in serialized.iter() {
        if original.get(key) != Some(value) {
            return false;
        }
    }
    true
}

fn print_results(
    file_name: &str,
    success_count: usize,
    total_abilities: usize,
    errors: &[RoundTripError],
) {
    println!("\n========================================");
    println!("{} Round-Trip Validation Results", file_name);
    println!("========================================\n");
    println!("Total abilities: {}", total_abilities);
    println!("Successfully round-tripped: {}", success_count);
    println!("Failed round-trip: {}\n", errors.len());

    if errors.is_empty() {
        println!("All abilities round-tripped successfully!");
    } else {
        println!("Round-Trip Failures:\n");
        println!("{}", "=".repeat(80));

        for (i, error) in errors.iter().enumerate() {
            println!("\nFailure #{}", i + 1);
            println!("{}", "-".repeat(80));
            println!("Dreamwell: {}", error.dreamwell_name);
            println!("\nOriginal Text:");
            println!("  {}", error.ability_text.replace('\n', "\n  "));

            if !error.variables.is_empty() {
                println!("\nOriginal Variables:");
                println!("  {}", error.variables.replace('\n', "\n  "));
            }

            match &error.error_type {
                RoundTripErrorType::TextMismatch => {
                    println!("\nError: Text mismatch");
                    println!("\nSerialized Text:");
                    println!("  {}", error.serialized_text.replace('\n', "\n  "));
                }
                RoundTripErrorType::VariableMismatch => {
                    println!("\nError: Variable mismatch");
                    println!("\nSerialized Variables:");
                    println!("  {}", error.serialized_variables);
                }
                RoundTripErrorType::ParseError(msg) => {
                    println!("\nParse Error: {}", msg);
                }
            }
            println!("{}", "-".repeat(80));
        }
    }
    println!("\n========================================\n");
}
