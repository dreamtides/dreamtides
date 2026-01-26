use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata_types::{DerivedColumnConfig, SortConfig};
use crate::traits::{FileSystem, RealFileSystem};
use crate::validation::validation_rules::{ValidationRule, ValueType};

/// Parses the metadata.sort section from a TOML file using the real filesystem.
pub fn parse_sort_config_from_file(file_path: &str) -> Result<Option<SortConfig>, TvError> {
    parse_sort_config_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.sort section from a TOML file and returns the sort configuration.
pub fn parse_sort_config_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Option<SortConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_sort_config_from_content(&content, file_path)
}

/// Parses sort configuration from TOML content string.
pub fn parse_sort_config_from_content(
    content: &str,
    file_path: &str,
) -> Result<Option<SortConfig>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(None);
    };

    let Some(sort_value) = metadata.get("sort") else {
        return Ok(None);
    };

    let Some(sort_table) = sort_value.as_table() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.sort is not a table"
        );
        return Ok(None);
    };

    let column = match sort_table.get("column").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            tracing::warn!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                "metadata.sort missing 'column' field"
            );
            return Ok(None);
        }
    };

    let ascending = sort_table.get("ascending").and_then(|v| v.as_bool()).unwrap_or(true);

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        column = %column,
        ascending = ascending,
        "Parsed sort config from metadata"
    );

    Ok(Some(SortConfig { column, ascending }))
}

/// Parses the metadata.validation_rules section from a TOML file using the real filesystem.
pub fn parse_validation_rules_from_file(file_path: &str) -> Result<Vec<ValidationRule>, TvError> {
    parse_validation_rules_with_fs(&RealFileSystem, file_path)
}

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

/// Parses the metadata.derived_columns section from a TOML file using the real filesystem.
pub fn parse_derived_columns_from_file(file_path: &str) -> Result<Vec<DerivedColumnConfig>, TvError> {
    parse_derived_columns_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.derived_columns section from a TOML file.
pub fn parse_derived_columns_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<DerivedColumnConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_derived_columns_from_content(&content, file_path)
}

/// Parses derived column configurations from TOML content string.
pub fn parse_derived_columns_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<DerivedColumnConfig>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(Vec::new());
    };

    let Some(derived_columns_value) = metadata.get("derived_columns") else {
        return Ok(Vec::new());
    };

    let Some(columns_array) = derived_columns_value.as_array() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.derived_columns is not an array"
        );
        return Ok(Vec::new());
    };

    let mut configs = Vec::new();
    for (idx, col_value) in columns_array.iter().enumerate() {
        match parse_single_derived_column(col_value, file_path, idx) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml.metadata",
                    file_path = %file_path,
                    column_index = idx,
                    error = %e,
                    "Failed to parse derived column config, skipping"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        column_count = configs.len(),
        "Parsed derived column configs from metadata"
    );

    Ok(configs)
}

fn parse_single_derived_column(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<DerivedColumnConfig, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("derived_columns[{idx}] is not a table"),
    })?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("derived_columns[{idx}] missing 'name' field"),
        })?
        .to_string();

    let function = table
        .get("function")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("derived_columns[{idx}] missing 'function' field"),
        })?
        .to_string();

    let position = table.get("position").and_then(|v| v.as_integer()).map(|i| i as usize);

    let width = table
        .get("width")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32)
        .unwrap_or(100);

    let inputs = table
        .get("inputs")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Ok(DerivedColumnConfig { name, function, position, width, inputs })
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
        "type" => {
            let value_type = table
                .get("value_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TvError::MetadataCorrupt {
                    path: file_path.to_string(),
                    message: format!("validation_rules[{}] of type 'type' missing 'value_type' field", idx),
                })?;
            let value_type = parse_value_type(value_type, file_path, idx)?;
            Ok(ValidationRule::Type { column, value_type, message })
        }
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

fn parse_value_type(value_type: &str, file_path: &str, idx: usize) -> Result<ValueType, TvError> {
    match value_type {
        "string" => Ok(ValueType::String),
        "integer" => Ok(ValueType::Integer),
        "float" => Ok(ValueType::Float),
        "boolean" => Ok(ValueType::Boolean),
        other => Err(TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("validation_rules[{}] has unknown value_type '{}'", idx, other),
        }),
    }
}
