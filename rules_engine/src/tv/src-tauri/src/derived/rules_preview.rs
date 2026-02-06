use crate::derived::rlf_integration;
use crate::derived::style_tag_parser;

use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};

/// A derived function that renders rules text with variable substitution
/// and rich text styling.
///
/// Given `rules_text` containing RLF expressions and `variables`
/// containing key-value pairs, this function formats the text through
/// the RLF template system and parses HTML-like style tags to produce
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
        vec!["rules-text", "variables"]
    }

    fn compute(&self, inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        let rules_text = match inputs.get("rules-text") {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Null) | None => return DerivedResult::Text(String::new()),
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid rules_text type: expected string, got {}",
                    json_type_name(other)
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
                    "Invalid variables type: expected string, got {}",
                    json_type_name(other)
                ));
            }
        };

        let params = match rlf_integration::parse_variables(variables_text) {
            Ok(parsed) => parsed.to_rlf_params(),
            Err(e) => return DerivedResult::Error(format!("Variable parse error: {e}")),
        };

        let formatted = match rlf_integration::format_expression(rules_text, params) {
            Ok(s) => s,
            Err(e) => return DerivedResult::Error(format!("RLF error: {e}")),
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
