use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::toml::metadata_types::{ConditionalFormatRule, FormatCondition, FormatStyle};

/// Result of evaluating conditional formatting rules for a single cell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellFormatResult {
    pub row: usize,
    pub column: String,
    pub col_index: usize,
    pub style: FormatStyle,
}

/// Evaluates all conditional formatting rules against a table's data.
///
/// Returns a list of cell format results for cells where at least one
/// rule matched. Later rules in the list override earlier ones for the
/// same style property when multiple rules match the same cell.
pub fn evaluate_rules(
    rules: &[ConditionalFormatRule],
    headers: &[String],
    rows: &[Vec<serde_json::Value>],
) -> Vec<CellFormatResult> {
    let mut results: Vec<CellFormatResult> = Vec::new();

    for rule in rules {
        let Some(col_index) = headers.iter().position(|h| h == &rule.column) else {
            continue;
        };

        for (row_index, row) in rows.iter().enumerate() {
            let cell_value = row.get(col_index).cloned().unwrap_or(serde_json::Value::Null);

            if evaluate_condition(&rule.condition, &cell_value) {
                if let Some(existing) =
                    results.iter_mut().find(|r| r.row == row_index && r.col_index == col_index)
                {
                    merge_styles(&mut existing.style, &rule.style);
                } else {
                    results.push(CellFormatResult {
                        row: row_index,
                        column: rule.column.clone(),
                        col_index,
                        style: rule.style.clone(),
                    });
                }
            }
        }
    }

    results
}

/// Evaluates a single condition against a cell value.
pub fn evaluate_condition(condition: &FormatCondition, cell_value: &serde_json::Value) -> bool {
    match condition {
        FormatCondition::Equals(expected) => values_equal(cell_value, expected),
        FormatCondition::Contains(substring) => {
            let cell_str = value_to_string(cell_value);
            cell_str.contains(substring.as_str())
        }
        FormatCondition::GreaterThan(threshold) => {
            value_as_f64(cell_value).is_some_and(|n| n > *threshold)
        }
        FormatCondition::LessThan(threshold) => {
            value_as_f64(cell_value).is_some_and(|n| n < *threshold)
        }
        FormatCondition::IsEmpty => is_empty(cell_value),
        FormatCondition::NotEmpty => !is_empty(cell_value),
        FormatCondition::Matches(pattern) => {
            let Ok(regex) = Regex::new(pattern) else {
                tracing::warn!(
                    component = "tv.toml.conditional_formatting",
                    pattern = %pattern,
                    "Invalid regex pattern in conditional formatting rule"
                );
                return false;
            };
            regex.is_match(&value_to_string(cell_value))
        }
    }
}

fn values_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::String(s1), serde_json::Value::String(s2)) => s1 == s2,
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
            n1.as_f64() == n2.as_f64()
        }
        (serde_json::Value::Bool(b1), serde_json::Value::Bool(b2)) => b1 == b2,
        (serde_json::Value::String(s), serde_json::Value::Number(n)) => {
            s.parse::<f64>().ok().and_then(|f| n.as_f64().map(|nf| (f - nf).abs() < f64::EPSILON))
                == Some(true)
        }
        (serde_json::Value::Number(n), serde_json::Value::String(s)) => {
            s.parse::<f64>().ok().and_then(|f| n.as_f64().map(|nf| (f - nf).abs() < f64::EPSILON))
                == Some(true)
        }
        _ => a == b,
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn value_as_f64(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn is_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        _ => false,
    }
}

/// Merges a new style into an existing style, with new values overriding existing ones.
fn merge_styles(existing: &mut FormatStyle, new_style: &FormatStyle) {
    if new_style.background_color.is_some() {
        existing.background_color.clone_from(&new_style.background_color);
    }
    if new_style.font_color.is_some() {
        existing.font_color.clone_from(&new_style.font_color);
    }
    if new_style.bold.is_some() {
        existing.bold = new_style.bold;
    }
    if new_style.italic.is_some() {
        existing.italic = new_style.italic;
    }
    if new_style.underline.is_some() {
        existing.underline = new_style.underline;
    }
}
