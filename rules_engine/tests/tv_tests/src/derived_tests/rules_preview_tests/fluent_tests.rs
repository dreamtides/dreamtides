use std::collections::HashMap;

use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::fluent_integration::initialize_fluent_resource;
use tv_lib::derived::rules_preview::RulesPreviewFunction;

fn setup() {
    initialize_fluent_resource();
}

fn create_empty_context() -> LookupContext {
    LookupContext::new()
}

fn make_inputs(rules_text: &str, variables: &str) -> RowData {
    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!(rules_text));
    inputs.insert("variables".to_string(), serde_json::json!(variables));
    inputs
}

#[test]
fn test_variable_substitution() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Deal { $damage } damage.", "damage: 4");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "Deal 4 damage.", "Variable should be substituted in output");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_fluent_term_expansion() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Foresee}", "foresee: 3");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                full_text.contains("Foresee"),
                "Term expansion should contain 'Foresee': {full_text}"
            );
            assert!(
                full_text.contains("3"),
                "Term expansion should contain the variable value: {full_text}"
            );
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_missing_variable_error() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Use { $nonexistent } power.", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Fluent error"),
                "Missing variable should produce a Fluent error: {msg}"
            );
        }
        other => panic!("Expected Error for missing variable, got: {other:?}"),
    }
}

#[test]
fn test_invalid_fluent_syntax_error() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{ $x ->", "x: 1");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Fluent error") || msg.contains("parse"),
                "Invalid syntax should produce an error: {msg}"
            );
        }
        other => panic!("Expected Error for invalid syntax, got: {other:?}"),
    }
}

#[test]
fn test_numeric_variable_parsing() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs =
        make_inputs("Costs { $cost } energy, gives { $points } points.", "cost: 7\npoints: 2");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(
                full_text, "Costs 7 energy, gives 2 points.",
                "Numeric variables should be substituted correctly"
            );
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_multiline_expression() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Line one.\nLine two.\nLine three.", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.contains("Line one."), "Should contain first line: {full_text}");
            assert!(full_text.contains("Line two."), "Should contain second line: {full_text}");
            assert!(full_text.contains("Line three."), "Should contain third line: {full_text}");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_double_newline_expression() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs(
        "Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}",
        "cards: 2\ndiscards: 2\nreclaim: 2",
    );

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                !full_text.contains('\n'),
                "Preview should not contain newlines: {full_text:?}"
            );
            assert!(full_text.contains("Draw"), "Should contain 'Draw': {full_text}");
            assert!(full_text.contains("Reclaim"), "Should contain 'Reclaim': {full_text}");
        }
        DerivedResult::Error(msg) => panic!("Expected RichText, got Error: {msg}"),
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_single_newline_expression() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Line one.\nLine two.", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                !full_text.contains('\n'),
                "Preview should not contain newlines: {full_text:?}"
            );
            assert!(full_text.contains("Line one."), "Should contain first line: {full_text}");
            assert!(full_text.contains("Line two."), "Should contain second line: {full_text}");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}
