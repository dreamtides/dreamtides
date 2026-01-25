use regex::Regex;
use serde_json::Value;

use crate::validation::validation_rules::{ValidationResult, ValidationRule, ValueType};

pub fn validate(rule: &ValidationRule, value: &Value) -> ValidationResult {
    match rule {
        ValidationRule::Enum { column, allowed_values, message } => {
            validate_enum(column, allowed_values, message.as_deref(), value)
        }
        ValidationRule::Range { column, min, max, message } => {
            validate_range(column, *min, *max, message.as_deref(), value)
        }
        ValidationRule::Pattern { column, pattern, message } => {
            validate_pattern(column, pattern, message.as_deref(), value)
        }
        ValidationRule::Required { column, message } => {
            validate_required(column, message.as_deref(), value)
        }
        ValidationRule::Type { column, value_type, message } => {
            validate_type(column, *value_type, message.as_deref(), value)
        }
    }
}

pub fn validate_all(rules: &[ValidationRule], column: &str, value: &Value) -> Vec<ValidationResult> {
    rules.iter().filter(|r| r.column() == column).map(|r| validate(r, value)).collect()
}

pub fn is_valid(results: &[ValidationResult]) -> bool {
    results.iter().all(|r| r.valid)
}

pub fn first_error(results: &[ValidationResult]) -> Option<&ValidationResult> {
    results.iter().find(|r| !r.valid)
}

fn validate_enum(
    column: &str,
    allowed_values: &[String],
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, "enum");
    }

    let string_value = match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    if allowed_values.contains(&string_value) {
        ValidationResult::success(column, "enum")
    } else {
        let message = custom_message.map(String::from).unwrap_or_else(|| {
            format!(
                "Value '{}' is not allowed. Must be one of: {}",
                string_value,
                allowed_values.join(", ")
            )
        });
        ValidationResult::failure(column, "enum", message)
    }
}

fn validate_range(
    column: &str,
    min: Option<f64>,
    max: Option<f64>,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, "range");
    }

    let number = match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };

    let Some(num) = number else {
        let message = custom_message
            .map(String::from)
            .unwrap_or_else(|| format!("Value '{}' is not a valid number", value));
        return ValidationResult::failure(column, "range", message);
    };

    if let Some(min_val) = min {
        if num < min_val {
            let message = custom_message
                .map(String::from)
                .unwrap_or_else(|| format!("Value {} is less than minimum {}", num, min_val));
            return ValidationResult::failure(column, "range", message);
        }
    }

    if let Some(max_val) = max {
        if num > max_val {
            let message = custom_message
                .map(String::from)
                .unwrap_or_else(|| format!("Value {} is greater than maximum {}", num, max_val));
            return ValidationResult::failure(column, "range", message);
        }
    }

    ValidationResult::success(column, "range")
}

fn validate_pattern(
    column: &str,
    pattern: &str,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, "pattern");
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
                "pattern",
                format!("Invalid regex pattern '{}': {}", pattern, e),
            );
        }
    };

    if regex.is_match(&string_value) {
        ValidationResult::success(column, "pattern")
    } else {
        let message = custom_message
            .map(String::from)
            .unwrap_or_else(|| format!("Value '{}' does not match pattern '{}'", string_value, pattern));
        ValidationResult::failure(column, "pattern", message)
    }
}

fn validate_required(column: &str, custom_message: Option<&str>, value: &Value) -> ValidationResult {
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
        ValidationResult::failure(column, "required", message)
    } else {
        ValidationResult::success(column, "required")
    }
}

fn validate_type(
    column: &str,
    expected_type: ValueType,
    custom_message: Option<&str>,
    value: &Value,
) -> ValidationResult {
    if value.is_null() {
        return ValidationResult::success(column, "type");
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
        ValidationResult::success(column, "type")
    } else {
        let message = custom_message.map(String::from).unwrap_or_else(|| {
            let type_name = match expected_type {
                ValueType::String => "string",
                ValueType::Integer => "integer",
                ValueType::Float => "number",
                ValueType::Boolean => "boolean",
            };
            format!("Value '{}' is not a valid {}", value, type_name)
        });
        ValidationResult::failure(column, "type", message)
    }
}
