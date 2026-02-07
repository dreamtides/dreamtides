//! Tests that evaluate all cards' rules text through the TV viewer's RLF
//! pipeline, catching any evaluation errors (e.g. subtype variables not
//! being converted to Phrase values for the `:from` modifier).

use std::collections::HashMap;

use serde::Deserialize;
use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::rules_preview::RulesPreviewFunction;

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

struct EvalError {
    card_name: String,
    rules_text: String,
    variables: String,
    error: String,
}

fn evaluate_all_cards(file_path: &str, table_key: &str) -> (usize, Vec<EvalError>) {
    let toml_content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read {file_path}: {e}"));
    let cards: Vec<Card> = {
        let table: toml::Value = toml::from_str(&toml_content)
            .unwrap_or_else(|e| panic!("Failed to parse {file_path}: {e}"));
        let array = table
            .get(table_key)
            .unwrap_or_else(|| panic!("Missing '{table_key}' table in {file_path}"));
        array
            .as_array()
            .unwrap_or_else(|| panic!("'{table_key}' should be an array in {file_path}"))
            .iter()
            .map(|v| v.clone().try_into().unwrap())
            .collect()
    };

    let function = RulesPreviewFunction::new();
    let context = LookupContext::new();
    let mut success_count = 0;
    let mut errors = Vec::new();

    for card in &cards {
        let Some(rules_text) = &card.rules_text else {
            continue;
        };

        let variables = card.variables.as_deref().unwrap_or("");

        let mut inputs: RowData = HashMap::new();
        inputs.insert("rules-text".to_string(), serde_json::json!(rules_text));
        inputs.insert("variables".to_string(), serde_json::json!(variables));

        let result = function.compute(&inputs, &context);
        match result {
            DerivedResult::Error(msg) => {
                errors.push(EvalError {
                    card_name: card.name.clone(),
                    rules_text: rules_text.clone(),
                    variables: variables.to_string(),
                    error: msg,
                });
            }
            _ => {
                success_count += 1;
            }
        }
    }

    (success_count, errors)
}

fn print_and_assert(file_name: &str, success_count: usize, errors: &[EvalError]) {
    let total = success_count + errors.len();
    println!("\n========================================");
    println!("{file_name} RLF Evaluation Results");
    println!("========================================\n");
    println!("Total cards with rules text: {total}");
    println!("Successfully evaluated: {success_count}");
    println!("Failed: {}\n", errors.len());

    if !errors.is_empty() {
        for (i, error) in errors.iter().enumerate() {
            println!("Failure #{}", i + 1);
            println!("{}", "-".repeat(60));
            println!("Card: {}", error.card_name);
            println!("Rules text: {}", error.rules_text);
            println!("Variables: {}", error.variables);
            println!("Error: {}", error.error);
            println!("{}", "-".repeat(60));
        }

        panic!("\n{} cards failed RLF evaluation in {file_name}", errors.len());
    }
}

#[test]
fn test_all_cards_toml_rlf_eval() {
    let (success, errors) = evaluate_all_cards("../../tabula/cards.toml", "cards");
    print_and_assert("cards.toml", success, &errors);
}

#[test]
fn test_all_test_cards_toml_rlf_eval() {
    let (success, errors) = evaluate_all_cards("../../tabula/test-cards.toml", "test-cards");
    print_and_assert("test-cards.toml", success, &errors);
}

#[test]
fn test_all_dreamwell_toml_rlf_eval() {
    let (success, errors) = evaluate_all_cards("../../tabula/dreamwell.toml", "dreamwell");
    print_and_assert("dreamwell.toml", success, &errors);
}

#[test]
fn test_all_test_dreamwell_toml_rlf_eval() {
    let (success, errors) =
        evaluate_all_cards("../../tabula/test-dreamwell.toml", "test-dreamwell");
    print_and_assert("test-dreamwell.toml", success, &errors);
}
