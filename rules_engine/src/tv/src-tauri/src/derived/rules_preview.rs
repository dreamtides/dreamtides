use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use crate::derived::fluent_integration;
use crate::derived::style_tag_parser;

/// A derived function that renders rules text with variable substitution
/// and rich text styling.
///
/// Given `rules_text` containing Fluent expressions and `variables`
/// containing key-value pairs, this function formats the text through
/// the Fluent template system and parses HTML-like style tags to produce
/// styled rich text output for Univer rendering.
pub struct RulesPreviewFunction;

impl RulesPreviewFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RulesPreviewFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivedFunction for RulesPreviewFunction {
    fn name(&self) -> &'static str {
        "rules_preview"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["rules_text", "variables"]
    }

    fn compute(&self, inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        let rules_text = match inputs.get("rules_text") {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Null) | None => return DerivedResult::Text(String::new()),
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid rules_text type: expected string, got {}", json_type_name(other)
                ));
            }
        };

        if rules_text.is_empty() {
            return DerivedResult::Text(String::new());
        }

        let variables_text = match inputs.get("variables") {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Null) | None => "",
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid variables type: expected string, got {}", json_type_name(other)
                ));
            }
        };

        let args = match fluent_integration::parse_variables(variables_text) {
            Ok(parsed) => parsed.to_fluent_args(),
            Err(e) => return DerivedResult::Error(format!("Variable parse error: {e}")),
        };

        let formatted = match fluent_integration::format_expression(rules_text, &args) {
            Ok(s) => s,
            Err(e) => return DerivedResult::Error(format!("Fluent error: {e}")),
        };

        let spans = style_tag_parser::parse_style_tags(&formatted);
        if spans.is_empty() {
            return DerivedResult::Text(String::new());
        }

        DerivedResult::RichText(spans)
    }

    fn is_async(&self) -> bool {
        false
    }
}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
