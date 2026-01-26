use regex::Regex;
use serde_json::Value;

use crate::validation::validation_rules::{ValidationResult, ValidationRule, ValueType};

/// Validates a value against a single validation rule.
pub fn validate(rule: &ValidationRule, value: &Value) -> ValidationResult {
    let column = rule.column();
    let rule_type = rule.rule_type_name();
    match rule {
        ValidationRule::Enum { allowed_values, message, .. } => {
            validate_enum(column, rule_type, allowed_values, message.as_deref(), value)
        }
        ValidationRule::Range { min, max, message, .. } => {
            validate_range(column, rule_type, *min, *max, message.as_deref(), value)
        }
        ValidationRule::Pattern { pattern, message, .. } => {
            validate_pattern(column, rule_type, pattern, message.as_deref(), value)
        }
        ValidationRule::Required { message, .. } => {
            validate_required(column, rule_type, message.as_deref(), value)
        }
        ValidationRule::Type { value_type, message, .. } => {
            validate_type(column, rule_type, *value_type, message.as_deref(), value)
        }
    }
}

/// Validates a value against all rules that apply to the given column.
pub fn validate_all(rules: &[ValidationRule], column: &str, value: &Value) -> Vec<ValidationResult> {
    rules.iter().filter(|r| r.column() == column).map(|r| validate(r, value)).collect()
}

/// Returns true if all validation results are valid.
pub fn is_valid(results: &[ValidationResult]) -> bool {
    results.iter().all(|r| r.valid)
}

/// Returns the first validation failure, if any.
pub fn first_error(results: &[ValidationResult]) -> Option<&ValidationResult> {
    results.iter().find(|r| !r.valid)
}

fn validate_enum(
    column: &str,
    rule_type: &str,
    allowed_values: &[String],
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, rule_type);
    }

    let string_value = match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    if allowed_values.contains(&string_value) {
        ValidationResult::success(column, rule_type)
    } else {
        let message = custom_message.map(String::from).unwrap_or_else(|| {
            format!(
                "Value '{}' is not allowed. Must be one of: {}",
                string_value,
                allowed_values.join(", ")
            )
        });
        ValidationResult::failure(column, rule_type, message)
    }
}

fn validate_range(
    column: &str,
    rule_type: &str,
    min: Option<f64>,
    max: Option<f64>,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, rule_type);
    }

    let number = match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };

    let Some(num) = number else {
        let message = custom_message
            .map(String::from)
            .unwrap_or_else(|| format!("Value '{value}' is not a valid number"));
        return ValidationResult::failure(column, rule_type, message);
    };

    if let Some(min_val) = min {
        if num < min_val {
            let message = custom_message
                .map(String::from)
                .unwrap_or_else(|| format!("Value {num} is less than minimum {min_val}"));
            return ValidationResult::failure(column, rule_type, message);
        }
    }

    if let Some(max_val) = max {
        if num > max_val {
            let message = custom_message
                .map(String::from)
                .unwrap_or_else(|| format!("Value {num} is greater than maximum {max_val}"));
            return ValidationResult::failure(column, rule_type, message);
        }
    }

    ValidationResult::success(column, rule_type)
}

fn validate_pattern(
    column: &str,
    rule_type: &str,
    pattern: &str,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, rule_type);
    }

    let string_value = match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            return ValidationResult::failure(
                column,
                rule_type,
                format!("Invalid regex pattern '{pattern}': {e}"),
            );
        }
    };

    if regex.is_match(&string_value) {
        ValidationResult::success(column, rule_type)
    } else {
        let message = custom_message
            .map(String::from)
            .unwrap_or_else(|| format!("Value '{string_value}' does not match pattern '{pattern}'"));
        ValidationResult::failure(column, rule_type, message)
    }
}

fn validate_required(
    column: &str,
    rule_type: &str,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    let is_empty = match value {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => false,
    };

    if is_empty {
        let message =
            custom_message.map(String::from).unwrap_or_else(|| "This field is required".to_string());
        ValidationResult::failure(column, rule_type, message)
    } else {
        ValidationResult::success(column, rule_type)
    }
}

fn validate_type(
    column: &str,
    rule_type: &str,
    expected_type: ValueType,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, rule_type);
    }

    let type_matches = match expected_type {
        ValueType::String => value.is_string(),
        ValueType::Integer => {
            value.is_i64() || value.is_u64() || matches!(value.as_f64(), Some(f) if f.fract() == 0.0)
        }
        ValueType::Float => value.is_f64() || value.is_i64() || value.is_u64(),
        ValueType::Boolean => value.is_boolean(),
    };

    if type_matches {
        ValidationResult::success(column, rule_type)
    } else {
        let message = custom_message
            .map(String::from)
            .unwrap_or_else(|| format!("Value '{value}' is not a valid {expected_type}"));
        ValidationResult::failure(column, rule_type, message)
    }
}
