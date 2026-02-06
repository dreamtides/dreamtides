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

fn extract_rich_text(result: DerivedResult) -> Vec<tv_lib::derived::derived_types::StyledSpan> {
    match result {
        DerivedResult::RichText(spans) => spans,
        other => panic!("Expected RichText, got: {other:?}"),
    }
}

fn full_text(spans: &[tv_lib::derived::derived_types::StyledSpan]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn test_appendix_g_keyword_foresee_example() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Foresee(foresee)}", "foresee: 3");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("foresee"), "Should contain keyword text: {text}");
    assert!(text.contains("3"), "Should contain the variable value: {text}");

    let colored_span = spans.iter().find(|s| s.color.is_some());
    assert!(colored_span.is_some(), "Should find a colored span: {spans:?}");
    assert_eq!(
        colored_span.unwrap().color.as_deref(),
        Some("AA00FF"),
        "Foresee keyword should use keyword color AA00FF"
    );
}

#[test]
fn test_trigger_materialized_phrase_expansion() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{materialized} Gain {e} energy.", "e: 2");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("Materialized"), "Should contain trigger name: {text}");
    assert!(text.contains("2"), "Should contain variable value: {text}");

    let bold_span = spans.iter().find(|s| s.text.contains("Materialized"));
    assert!(bold_span.is_some(), "Should find Materialized span: {spans:?}");
    assert!(bold_span.unwrap().bold, "Trigger text should be bold: {spans:?}");
}

#[test]
fn test_italic_style_tags() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<i>italic text</i> normal", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    assert!(spans.len() >= 2, "Should have at least 2 spans: {spans:?}");

    let italic_span = spans.iter().find(|s| s.text.contains("italic text"));
    assert!(italic_span.is_some(), "Should find italic span: {spans:?}");
    assert!(italic_span.unwrap().italic, "Should be italic: {spans:?}");
    assert!(!italic_span.unwrap().bold, "Should not be bold: {spans:?}");

    let normal_span = spans.iter().find(|s| s.text.contains("normal"));
    assert!(normal_span.is_some(), "Should find normal span: {spans:?}");
    assert!(!normal_span.unwrap().italic, "Should not be italic: {spans:?}");
}

#[test]
fn test_underline_style_tags() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<u>underlined</u> plain", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let underlined = spans.iter().find(|s| s.text.contains("underlined"));
    assert!(underlined.is_some(), "Should find underlined span: {spans:?}");
    assert!(underlined.unwrap().underline, "Should be underlined: {spans:?}");

    let plain = spans.iter().find(|s| s.text.contains("plain"));
    assert!(plain.is_some(), "Should find plain span: {spans:?}");
    assert!(!plain.unwrap().underline, "Should not be underlined: {spans:?}");
}

#[test]
fn test_unicode_content_in_output() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Hello world", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert_eq!(text, "Hello world", "Unicode should be preserved");
}

#[test]
fn test_unicode_variable_value() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Name: {name}", "name: Dragon");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("Dragon"), "Unicode variable value should be preserved: {text}");
}

#[test]
fn test_malformed_tag_passed_through_as_literal() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<invalid>text</invalid>", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("<invalid>"), "Invalid tag should be passed through: {text}");
    assert!(text.contains("</invalid>"), "Invalid closing tag should be passed through: {text}");
}

#[test]
fn test_combined_variable_substitution_and_styling() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs =
        make_inputs("<b>Deal</b> {damage} <color=#FF0000>fire</color> damage.", "damage: 5");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("Deal"), "Should contain bold text: {text}");
    assert!(text.contains("5"), "Should substitute variable: {text}");
    assert!(text.contains("fire"), "Should contain colored text: {text}");
    assert!(text.contains("damage."), "Should contain trailing text: {text}");

    let bold_span = spans.iter().find(|s| s.text.contains("Deal"));
    assert!(bold_span.unwrap().bold, "Deal should be bold: {spans:?}");

    let colored_span = spans.iter().find(|s| s.text.contains("fire"));
    assert_eq!(
        colored_span.unwrap().color.as_deref(),
        Some("FF0000"),
        "Fire should be red: {spans:?}"
    );
}

#[test]
fn test_keyword_dissolve_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Dissolve} target character.", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("dissolve"), "Should contain keyword text: {text}");

    let keyword_span = spans.iter().find(|s| s.color.as_deref() == Some("AA00FF"));
    assert!(keyword_span.is_some(), "Should find keyword colored span: {spans:?}");
}

#[test]
fn test_keyword_reclaim_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Reclaim} a character.", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("reclaim"), "Should contain Reclaim keyword: {text}");

    let keyword_span = spans.iter().find(|s| s.color.as_deref() == Some("AA00FF"));
    assert!(keyword_span.is_some(), "Reclaim should use keyword color: {spans:?}");
}

#[test]
fn test_keyword_prevent_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Prevent} the next event.", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("prevent"), "Should contain Prevent keyword: {text}");
}

#[test]
fn test_fast_keyword_produces_bold() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Fast}", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("fast"), "Should contain Fast text: {text}");

    let fast_span = spans.iter().find(|s| s.text.to_lowercase().contains("fast"));
    assert!(fast_span.unwrap().bold, "Fast keyword should be bold: {spans:?}");
}

#[test]
fn test_energy_variable_rendering() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{energy(e)}", "e: 3");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("3"), "Should contain energy value: {text}");
}

#[test]
fn test_to_frontend_value_for_rich_text_result() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<b>Bold</b> text", "");

    let result = function.compute(&inputs, &context);
    let frontend = result.to_frontend_value();

    assert_eq!(frontend["type"], "richText", "Should be richText type");
    assert!(frontend["value"]["p"].is_array(), "Should have paragraphs array");
    assert!(!frontend["value"]["p"][0]["ts"].is_null(), "Should have text runs");
}

#[test]
fn test_to_frontend_value_for_empty_text_result() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("", "");

    let result = function.compute(&inputs, &context);
    let frontend = result.to_frontend_value();

    assert_eq!(frontend["type"], "text", "Empty input should produce text type");
    assert_eq!(frontend["value"], "", "Empty input should produce empty value");
}

#[test]
fn test_to_frontend_value_for_error_result() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!(42));

    let result = function.compute(&inputs, &context);
    let frontend = result.to_frontend_value();

    assert_eq!(frontend["type"], "error", "Invalid input should produce error type");
    assert!(
        frontend["value"].as_str().unwrap().contains("Invalid rules_text type"),
        "Error should describe the problem"
    );
}

#[test]
fn test_default_trait_creates_function() {
    let function = RulesPreviewFunction::default();
    assert_eq!(function.name(), "rules_preview");
    assert_eq!(function.input_keys(), vec!["rules-text", "variables"]);
    assert!(!function.is_async());
}

#[test]
fn test_whitespace_only_variables_treated_as_empty() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Plain text.", "   \n   \n   ");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert_eq!(text, "Plain text.");
}

#[test]
fn test_all_style_types_combined() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("<b><i><u><color=#FF0000>fully styled</color></u></i></b>", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    assert_eq!(spans.len(), 1, "Should be single combined span: {spans:?}");
    let span = &spans[0];
    assert_eq!(span.text, "fully styled");
    assert!(span.bold, "Should be bold");
    assert!(span.italic, "Should be italic");
    assert!(span.underline, "Should be underlined");
    assert_eq!(span.color.as_deref(), Some("FF0000"), "Should be red");
}

#[test]
fn test_multiple_style_sections() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs =
        make_inputs("<b>bold</b> <i>italic</i> <u>underline</u> <color=#0000FF>blue</color>", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    assert!(spans.len() >= 4, "Should have at least 4 styled sections: {spans:?}");

    let bold = spans.iter().find(|s| s.text.contains("bold"));
    assert!(bold.unwrap().bold, "Should be bold");

    let italic = spans.iter().find(|s| s.text.contains("italic"));
    assert!(italic.unwrap().italic, "Should be italic");

    let underline = spans.iter().find(|s| s.text.contains("underline"));
    assert!(underline.unwrap().underline, "Should be underlined");

    let blue = spans.iter().find(|s| s.text.contains("blue"));
    assert_eq!(blue.unwrap().color.as_deref(), Some("0000FF"), "Should be blue");
}

#[test]
fn test_choose_one_phrase() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{choose_one}", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("Choose One"), "Should expand choose_one phrase: {text}");

    let bold_span = spans.iter().find(|s| s.text.contains("Choose One"));
    assert!(bold_span.unwrap().bold, "Choose One should be bold: {spans:?}");
}

#[test]
fn test_kindle_with_variable() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{Kindle(k)}", "k: 2");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.to_lowercase().contains("kindle"), "Should contain Kindle keyword: {text}");
    assert!(text.contains("2"), "Should contain variable value: {text}");

    let keyword_span = spans.iter().find(|s| s.color.as_deref() == Some("AA00FF"));
    assert!(keyword_span.is_some(), "Kindle should use keyword color: {spans:?}");
}

#[test]
fn test_complex_card_text_with_multiple_phrases() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("{materialized} {Foresee(foresee)}. Draw a card.", "foresee: 3");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert!(text.contains("Materialized"), "Should contain trigger: {text}");
    assert!(text.to_lowercase().contains("foresee"), "Should contain keyword: {text}");
    assert!(text.contains("3"), "Should contain foresee value: {text}");
    assert!(text.contains("Draw a card"), "Should contain trailing text: {text}");
}

#[test]
fn test_array_type_rules_text_returns_error() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!([1, 2, 3]));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("array"), "Error should mention array type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_object_type_rules_text_returns_error() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!({"key": "value"}));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("object"), "Error should mention object type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_boolean_type_rules_text_returns_error() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!(true));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("boolean"), "Error should mention boolean type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_array_type_variables_returns_error() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("rules-text".to_string(), serde_json::json!("text"));
    inputs.insert("variables".to_string(), serde_json::json!(["a", "b"]));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("array"), "Error should mention array type: {msg}");
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

#[test]
fn test_rich_text_spans_preserve_text_content() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Start <b>middle</b> end.", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert_eq!(text, "Start middle end.", "All text content should be preserved");
}

#[test]
fn test_special_characters_in_text() {
    let function = RulesPreviewFunction::new();
    let context = create_empty_context();
    let inputs = make_inputs("Costs & benefits: 50% off!", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert_eq!(text, "Costs & benefits: 50% off!");
}

#[test]
fn test_lookup_context_not_used() {
    let function = RulesPreviewFunction::new();
    let mut context = LookupContext::new();
    let mut table_data = HashMap::new();
    let mut row = HashMap::new();
    row.insert("name".to_string(), serde_json::json!("Test Card"));
    table_data.insert("card-001".to_string(), row);
    context.add_table("cards", table_data);

    let inputs = make_inputs("Simple text.", "");

    let spans = extract_rich_text(function.compute(&inputs, &context));
    let text = full_text(&spans);
    assert_eq!(text, "Simple text.", "Context should not affect output");
}
