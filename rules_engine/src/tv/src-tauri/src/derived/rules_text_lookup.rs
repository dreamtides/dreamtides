use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use crate::derived::rlf_integration;
use crate::derived::style_tag_parser;

/// A derived function that looks up rules text via cross-table card ID
/// references and renders it with variable substitution and rich text styling.
pub struct RulesTextLookupFunction;

impl RulesTextLookupFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RulesTextLookupFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivedFunction for RulesTextLookupFunction {
    fn name(&self) -> &'static str {
        "rules_text_lookup"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["card_id"]
    }

    fn compute(&self, inputs: &RowData, context: &LookupContext) -> DerivedResult {
        let card_id_value = inputs.get("card-id").or_else(|| inputs.get("card_id"));

        let card_id = match card_id_value {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Null) | None => {
                return DerivedResult::Text(String::new());
            }
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid card_id type: expected string, got {}",
                    json_type_name(other)
                ));
            }
        };

        if card_id.is_empty() {
            return DerivedResult::Text(String::new());
        }

        match context.lookup_by_id_any_table(card_id) {
            Some((_table_name, row_data)) => {
                let rules_text = match row_data.get("rules-text") {
                    Some(serde_json::Value::String(s)) => s.as_str(),
                    Some(serde_json::Value::Null) | None => {
                        return DerivedResult::Text(String::new());
                    }
                    Some(_) => return DerivedResult::Text(String::new()),
                };

                if rules_text.is_empty() {
                    return DerivedResult::Text(String::new());
                }

                let variables_text = match row_data.get("variables") {
                    Some(serde_json::Value::String(s)) => s.as_str(),
                    _ => "",
                };

                let params = match rlf_integration::parse_variables(variables_text) {
                    Ok(parsed) => parsed.to_rlf_params(),
                    Err(e) => {
                        return DerivedResult::Error(format!("Variable parse error: {e}"))
                    }
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
            None => DerivedResult::Error(format!("Unknown Card: {card_id}")),
        }
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
