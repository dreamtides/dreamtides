mod fluent_tests;

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
    inputs.insert("rules_text".to_string(), serde_json::json!(rules_text));
    inputs.insert("variables".to_string(), serde_json::json!(variables));
    inputs
}

#[test]
fn test_function_name() {
    let function = RulesPreviewFunction::new();
    assert_eq!(function.name(), "rules_preview");
}

#[test]
fn test_input_keys() {
    let function = RulesPreviewFunction::new();
    assert_eq!(function.input_keys(), vec!["rules_text", "variables"]);
}

#[test]
fn test_is_not_async() {
    let function = RulesPreviewFunction::new();
    assert!(!function.is_async());
}

#[test]
fn test_simple_variable_substitution() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Gain { $e } energy.", "e: 3");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "Gain 3 energy.");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_empty_rules_text() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("", "");

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_null_rules_text() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules_text".to_string(), serde_json::Value::Null);

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_missing_rules_text_field() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs: RowData = HashMap::new();

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_invalid_rules_text_type() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules_text".to_string(), serde_json::json!(42));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("Invalid rules_text type"), "Error should mention type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_invalid_variables_type() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules_text".to_string(), serde_json::json!("Hello"));
    inputs.insert("variables".to_string(), serde_json::json!(42));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("Invalid variables type"), "Error should mention type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_null_variables_treated_as_empty() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules_text".to_string(), serde_json::json!("Hello world."));
    inputs.insert("variables".to_string(), serde_json::Value::Null);

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "Hello world.");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_missing_variables_field() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules_text".to_string(), serde_json::json!("Plain text output."));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "Plain text output.");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_malformed_variables_returns_error() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Hello", "invalid no colon");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Variable parse error"),
                "Error should mention parse error: {msg}"
            );
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_missing_fluent_variable_returns_error() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{ $missing_var }", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("Fluent error"), "Error should mention Fluent: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_bold_style_tags() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    // Directly provide text with bold tags (no Fluent processing needed)
    let inputs = make_inputs("<b>bold text</b> normal", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            assert!(spans.len() >= 2, "Should have at least 2 spans: {spans:?}");
            let bold_span = spans.iter().find(|s| s.text.contains("bold text"));
            assert!(bold_span.is_some(), "Should find bold text span: {spans:?}");
            assert!(bold_span.unwrap().bold, "Bold text should be bold: {spans:?}");

            let normal_span = spans.iter().find(|s| s.text.contains("normal"));
            assert!(normal_span.is_some(), "Should find normal text span: {spans:?}");
            assert!(!normal_span.unwrap().bold, "Normal text should not be bold: {spans:?}");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_color_style_tags() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<color=#FF0000>red text</color> plain", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let colored_span = spans.iter().find(|s| s.text.contains("red text"));
            assert!(colored_span.is_some(), "Should find colored text: {spans:?}");
            assert_eq!(
                colored_span.unwrap().color.as_deref(),
                Some("FF0000"),
                "Color should be FF0000: {spans:?}"
            );

            let plain_span = spans.iter().find(|s| s.text.contains("plain"));
            assert!(plain_span.is_some(), "Should find plain text: {spans:?}");
            assert!(
                plain_span.unwrap().color.is_none(),
                "Plain text should have no color: {spans:?}"
            );
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_nested_style_tags() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<color=#F57F17><b><u>Figment</u></b></color>", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            assert_eq!(spans.len(), 1, "Should have exactly one span: {spans:?}");
            let span = &spans[0];
            assert_eq!(span.text, "Figment");
            assert!(span.bold, "Should be bold");
            assert!(span.underline, "Should be underlined");
            assert_eq!(span.color.as_deref(), Some("F57F17"), "Should have color F57F17");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_multiple_variables() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{ $a } and { $b }", "a: hello\nb: world");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "hello and world");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_numeric_variable_substitution() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Draw { $n } cards.", "n: 5");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(full_text, "Draw 5 cards.");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_plain_text_no_styling() {
    setup();
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("No styling here.", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            assert_eq!(spans.len(), 1, "Should be a single span: {spans:?}");
            let span = &spans[0];
            assert_eq!(span.text, "No styling here.");
            assert!(!span.bold);
            assert!(!span.italic);
            assert!(!span.underline);
            assert!(span.color.is_none());
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}
