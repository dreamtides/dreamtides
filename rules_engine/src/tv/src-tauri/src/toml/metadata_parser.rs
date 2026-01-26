use std::path::Path;

use crate::error::error_types::TvError;
use crate::traits::FileSystem;
use crate::validation::validation_rules::{ValidationRule, ValueType};

/// Parses the metadata.validation_rules section from a TOML file and returns
/// a list of ValidationRule instances.
pub fn parse_validation_rules_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<ValidationRule>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_validation_rules_from_content(&content, file_path)
}

/// Parses validation rules from TOML content string.
pub fn parse_validation_rules_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<ValidationRule>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(Vec::new());
    };

    let Some(validation_rules_array) = metadata.get("validation_rules") else {
        return Ok(Vec::new());
    };

    let Some(rules_array) = validation_rules_array.as_array() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.validation_rules is not an array"
        );
        return Ok(Vec::new());
    };

    let mut rules = Vec::new();
    for (idx, rule_value) in rules_array.iter().enumerate() {
        match parse_single_rule(rule_value, file_path, idx) {
            Ok(rule) => rules.push(rule),
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml.metadata",
                    file_path = %file_path,
                    rule_index = idx,
                    error = %e,
                    "Failed to parse validation rule, skipping"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        rule_count = rules.len(),
        "Parsed validation rules from metadata"
    );

    Ok(rules)
}

fn parse_single_rule(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<ValidationRule, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("validation_rules[{}] is not a table", idx),
    })?;

    let column = table
        .get("column")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("validation_rules[{}] missing 'column' field", idx),
        })?
        .to_string();

    let rule_type = table
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("validation_rules[{}] missing 'type' field", idx),
        })?;

    let message = table.get("message").and_then(|v| v.as_str()).map(String::from);

    match rule_type {
        "enum" => {
            let allowed_values = table
                .get("enum")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Ok(ValidationRule::Enum { column, allowed_values, message })
        }
        "range" => {
            let min = table.get("min").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
            let max = table.get("max").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
            Ok(ValidationRule::Range { column, min, max, message })
        }
        "pattern" => {
            let pattern = table
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TvError::MetadataCorrupt {
                    path: file_path.to_string(),
                    message: format!("validation_rules[{}] of type 'pattern' missing 'pattern' field", idx),
                })?
                .to_string();
            Ok(ValidationRule::Pattern { column, pattern, message })
        }
        "required" => Ok(ValidationRule::Required { column, message }),
        "string" => Ok(ValidationRule::Type { column, value_type: ValueType::String, message }),
        "integer" => Ok(ValidationRule::Type { column, value_type: ValueType::Integer, message }),
        "float" => Ok(ValidationRule::Type { column, value_type: ValueType::Float, message }),
        "boolean" => Ok(ValidationRule::Type { column, value_type: ValueType::Boolean, message }),
        other => Err(TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("validation_rules[{}] has unknown type '{}'", idx, other),
        }),
    }
}
