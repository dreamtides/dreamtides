use std::collections::HashMap;

use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::rules_preview::RulesPreviewFunction;

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
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Deal {damage} damage.", "damage: 4");

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
fn test_rlf_phrase_expansion() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Foresee(foresee)}", "foresee: 3");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                full_text.to_lowercase().contains("foresee"),
                "Phrase expansion should contain 'foresee': {full_text}"
            );
            assert!(
                full_text.contains("3"),
                "Phrase expansion should contain the variable value: {full_text}"
            );
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_missing_variable_error() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{nonexistent_phrase_xyz}", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("RLF error"), "Missing phrase should produce an RLF error: {msg}");
        }
        other => panic!("Expected Error for missing phrase, got: {other:?}"),
    }
}

#[test]
fn test_numeric_variable_parsing() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs =
        make_inputs("Costs {energy(cost)} energy, gives {points(pts)} points.", "cost: 7\npts: 2");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.contains("7"), "Should contain energy cost value 7: {full_text}");
            assert!(full_text.contains("2"), "Should contain points value 2: {full_text}");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_multiline_expression() {
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
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs(
        "Draw {cards(cards_count)}.\n\n{Reclaim_For_Cost(reclaim_cost)}",
        "cards_count: 2\nreclaim_cost: 2",
    );

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.contains("Draw"), "Should contain 'Draw': {full_text}");
            assert!(full_text.contains('\n'), "Should preserve newlines: {full_text:?}");
            assert!(
                full_text.to_lowercase().contains("reclaim"),
                "Should contain 'reclaim': {full_text}"
            );
        }
        DerivedResult::Error(msg) => panic!("Expected RichText, got Error: {msg}"),
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_subtype_variable_produces_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{subtype(subtype)}", "subtype: Warrior");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.contains("Warrior"), "Should contain subtype name: {full_text}");
        }
        DerivedResult::Error(msg) => {
            panic!("Subtype variable should produce Phrase, not error: {msg}")
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_subtype_variable_spirit_animal() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{subtype(subtype)}", "subtype: SpiritAnimal");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                full_text.contains("Spirit Animal"),
                "Should contain subtype name: {full_text}"
            );
        }
        DerivedResult::Error(msg) => {
            panic!("Spirit animal subtype should produce Phrase, not error: {msg}")
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_subtype_variable_with_article() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{@a subtype(subtype)}", "subtype: Warrior");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.to_lowercase().contains("a "), "Should contain article: {full_text}");
            assert!(full_text.contains("Warrior"), "Should contain subtype name: {full_text}");
        }
        DerivedResult::Error(msg) => {
            panic!("Subtype with article should produce Phrase, not error: {msg}")
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_subtype_variable_plural() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{@plural subtype(subtype)}", "subtype: Warrior");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                full_text.contains("Warriors"),
                "Should contain plural subtype name: {full_text}"
            );
        }
        DerivedResult::Error(msg) => {
            panic!("Subtype plural should produce Phrase, not error: {msg}")
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_figment_variable_produces_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{figment(figment)}", "figment: celestial");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(
                full_text.contains("Celestial"),
                "Should contain figment type name: {full_text}"
            );
        }
        DerivedResult::Error(msg) => {
            panic!("Figment variable should produce Phrase, not error: {msg}")
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

#[test]
fn test_single_newline_expression() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Line one.\nLine two.", "");

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::RichText(spans) => {
            let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
            assert!(full_text.contains("Line one."), "Should contain first line: {full_text}");
            assert!(full_text.contains('\n'), "Should preserve newlines: {full_text:?}");
            assert!(full_text.contains("Line two."), "Should contain second line: {full_text}");
        }
        other => panic!("Expected RichText, got: {other:?}"),
    }
}
