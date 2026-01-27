use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata_types::{
    ColumnConfig, ColumnFilter, ConditionalFormatRule, DerivedColumnConfig, FilterCondition,
    FilterConfig, FormatCondition, FormatStyle, RowConfig, RowHeight, SortConfig, TableStyle,
};
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

/// Parses the metadata.filter section from a TOML file using the real filesystem.
pub fn parse_filter_config_from_file(file_path: &str) -> Result<Option<FilterConfig>, TvError> {
    parse_filter_config_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.filter section from a TOML file.
pub fn parse_filter_config_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Option<FilterConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_filter_config_from_content(&content, file_path)
}

/// Parses filter configuration from TOML content string.
pub fn parse_filter_config_from_content(
    content: &str,
    file_path: &str,
) -> Result<Option<FilterConfig>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(None);
    };

    let Some(filter_value) = metadata.get("filter") else {
        return Ok(None);
    };

    let Some(filter_table) = filter_value.as_table() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.filter is not a table"
        );
        return Ok(None);
    };

    let active = filter_table.get("active").and_then(|v| v.as_bool()).unwrap_or(false);

    let filters = match filter_table.get("filters").and_then(|v| v.as_array()) {
        Some(arr) => {
            let mut result = Vec::new();
            for (idx, filter_value) in arr.iter().enumerate() {
                match parse_single_filter(filter_value, file_path, idx) {
                    Ok(filter) => result.push(filter),
                    Err(e) => {
                        tracing::warn!(
                            component = "tv.toml.metadata",
                            file_path = %file_path,
                            filter_index = idx,
                            error = %e,
                            "Failed to parse filter, skipping"
                        );
                    }
                }
            }
            result
        }
        None => Vec::new(),
    };

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        filter_count = filters.len(),
        active = active,
        "Parsed filter config from metadata"
    );

    Ok(Some(FilterConfig { filters, active }))
}

fn parse_single_filter(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<ColumnFilter, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("filter.filters[{idx}] is not a table"),
    })?;

    let column = table
        .get("column")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("filter.filters[{idx}] missing 'column' field"),
        })?
        .to_string();

    let condition_value = table.get("condition").ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("filter.filters[{idx}] missing 'condition' field"),
    })?;

    let condition_table = condition_value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("filter.filters[{idx}] 'condition' is not a table"),
    })?;

    let condition = parse_filter_condition(condition_table, file_path, idx)?;

    Ok(ColumnFilter { column, condition })
}

fn parse_filter_condition(
    table: &toml::map::Map<String, toml::Value>,
    file_path: &str,
    idx: usize,
) -> Result<FilterCondition, TvError> {
    if let Some(val) = table.get("contains") {
        let s = val.as_str().ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("filter.filters[{idx}].condition.contains is not a string"),
        })?;
        return Ok(FilterCondition::Contains(s.to_string()));
    }

    if let Some(val) = table.get("equals") {
        let json_val = toml_value_to_json(val);
        return Ok(FilterCondition::Equals(json_val));
    }

    if table.contains_key("min") || table.contains_key("max") {
        let min = table.get("min").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
        let max = table.get("max").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
        return Ok(FilterCondition::Range { min, max });
    }

    if let Some(val) = table.get("boolean") {
        let b = val.as_bool().ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("filter.filters[{idx}].condition.boolean is not a bool"),
        })?;
        return Ok(FilterCondition::Boolean(b));
    }

    Err(TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("filter.filters[{idx}].condition has no recognized condition type"),
    })
}

fn toml_value_to_json(val: &toml::Value) -> serde_json::Value {
    match val {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::json!(*i),
        toml::Value::Float(f) => serde_json::json!(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(t) => {
            let map: serde_json::Map<String, serde_json::Value> =
                t.iter().map(|(k, v)| (k.clone(), toml_value_to_json(v))).collect();
            serde_json::Value::Object(map)
        }
    }
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

    let url_template = table.get("url_template").and_then(|v| v.as_str()).map(String::from);

    Ok(DerivedColumnConfig { name, function, position, width, inputs, url_template })
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

/// Parses the metadata.table_style section from a TOML file using the real filesystem.
pub fn parse_table_style_from_file(file_path: &str) -> Result<Option<TableStyle>, TvError> {
    parse_table_style_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.table_style section from a TOML file.
pub fn parse_table_style_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Option<TableStyle>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_table_style_from_content(&content, file_path)
}

/// Parses table style configuration from TOML content string.
pub fn parse_table_style_from_content(
    content: &str,
    file_path: &str,
) -> Result<Option<TableStyle>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(None);
    };

    let Some(style_value) = metadata.get("table_style") else {
        return Ok(None);
    };

    let Some(style_table) = style_value.as_table() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.table_style is not a table"
        );
        return Ok(None);
    };

    let color_scheme = style_table.get("color_scheme").and_then(|v| v.as_str()).map(String::from);
    let show_row_stripes =
        style_table.get("show_row_stripes").and_then(|v| v.as_bool()).unwrap_or(true);
    let show_column_stripes =
        style_table.get("show_column_stripes").and_then(|v| v.as_bool()).unwrap_or(false);
    let header_bold = style_table.get("header_bold").and_then(|v| v.as_bool()).unwrap_or(true);
    let header_background =
        style_table.get("header_background").and_then(|v| v.as_str()).map(String::from);

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        color_scheme = ?color_scheme,
        show_row_stripes = show_row_stripes,
        "Parsed table style from metadata"
    );

    Ok(Some(TableStyle {
        color_scheme,
        show_row_stripes,
        show_column_stripes,
        header_bold,
        header_background,
    }))
}

/// Parses the metadata.conditional_formatting section from a TOML file using the real filesystem.
pub fn parse_conditional_formatting_from_file(
    file_path: &str,
) -> Result<Vec<ConditionalFormatRule>, TvError> {
    parse_conditional_formatting_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.conditional_formatting section from a TOML file.
pub fn parse_conditional_formatting_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<ConditionalFormatRule>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_conditional_formatting_from_content(&content, file_path)
}

/// Parses conditional formatting rules from TOML content string.
pub fn parse_conditional_formatting_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<ConditionalFormatRule>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(Vec::new());
    };

    let Some(cf_value) = metadata.get("conditional_formatting") else {
        return Ok(Vec::new());
    };

    let Some(cf_array) = cf_value.as_array() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.conditional_formatting is not an array"
        );
        return Ok(Vec::new());
    };

    let mut rules = Vec::new();
    for (idx, rule_value) in cf_array.iter().enumerate() {
        match parse_single_conditional_format_rule(rule_value, file_path, idx) {
            Ok(rule) => rules.push(rule),
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml.metadata",
                    file_path = %file_path,
                    rule_index = idx,
                    error = %e,
                    "Failed to parse conditional formatting rule, skipping"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        rule_count = rules.len(),
        "Parsed conditional formatting rules from metadata"
    );

    Ok(rules)
}

fn parse_single_conditional_format_rule(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<ConditionalFormatRule, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}] is not a table"),
    })?;

    let column = table
        .get("column")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("conditional_formatting[{idx}] missing 'column' field"),
        })?
        .to_string();

    let condition_value = table.get("condition").ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}] missing 'condition' field"),
    })?;

    let condition = parse_format_condition(condition_value, file_path, idx)?;

    let style_value = table.get("style").ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}] missing 'style' field"),
    })?;

    let style = parse_format_style(style_value, file_path, idx)?;

    Ok(ConditionalFormatRule { column, condition, style })
}

fn parse_format_condition(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<FormatCondition, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}].condition is not a table"),
    })?;

    if let Some(val) = table.get("equals") {
        return Ok(FormatCondition::Equals(toml_value_to_json(val)));
    }
    if let Some(val) = table.get("contains") {
        let s = val.as_str().ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("conditional_formatting[{idx}].condition.contains is not a string"),
        })?;
        return Ok(FormatCondition::Contains(s.to_string()));
    }
    if let Some(val) = table.get("greater_than") {
        let n = val.as_float().or_else(|| val.as_integer().map(|i| i as f64)).ok_or_else(|| {
            TvError::MetadataCorrupt {
                path: file_path.to_string(),
                message: format!("conditional_formatting[{idx}].condition.greater_than is not a number"),
            }
        })?;
        return Ok(FormatCondition::GreaterThan(n));
    }
    if let Some(val) = table.get("less_than") {
        let n = val.as_float().or_else(|| val.as_integer().map(|i| i as f64)).ok_or_else(|| {
            TvError::MetadataCorrupt {
                path: file_path.to_string(),
                message: format!("conditional_formatting[{idx}].condition.less_than is not a number"),
            }
        })?;
        return Ok(FormatCondition::LessThan(n));
    }
    if table.get("is_empty").is_some() {
        return Ok(FormatCondition::IsEmpty);
    }
    if table.get("not_empty").is_some() {
        return Ok(FormatCondition::NotEmpty);
    }
    if let Some(val) = table.get("matches") {
        let s = val.as_str().ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("conditional_formatting[{idx}].condition.matches is not a string"),
        })?;
        return Ok(FormatCondition::Matches(s.to_string()));
    }

    Err(TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}].condition has no recognized condition type"),
    })
}

fn parse_format_style(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<FormatStyle, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("conditional_formatting[{idx}].style is not a table"),
    })?;

    Ok(FormatStyle {
        background_color: table.get("background_color").and_then(|v| v.as_str()).map(String::from),
        font_color: table.get("font_color").and_then(|v| v.as_str()).map(String::from),
        bold: table.get("bold").and_then(|v| v.as_bool()),
        italic: table.get("italic").and_then(|v| v.as_bool()),
        underline: table.get("underline").and_then(|v| v.as_bool()),
    })
}

/// Parses the metadata.rows section from a TOML file using the real filesystem.
pub fn parse_row_config_from_file(file_path: &str) -> Result<Option<RowConfig>, TvError> {
    parse_row_config_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.rows section from a TOML file.
pub fn parse_row_config_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Option<RowConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_row_config_from_content(&content, file_path)
}

/// Parses row configuration from TOML content string.
pub fn parse_row_config_from_content(
    content: &str,
    file_path: &str,
) -> Result<Option<RowConfig>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(None);
    };

    let Some(rows_value) = metadata.get("rows") else {
        return Ok(None);
    };

    let Some(rows_table) = rows_value.as_table() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.rows is not a table"
        );
        return Ok(None);
    };

    let header_height = rows_table
        .get("header_height")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32);

    let default_height = rows_table
        .get("default_height")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32);

    let frozen_rows = rows_table
        .get("frozen_rows")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32);

    let heights = match rows_table.get("heights").and_then(|v| v.as_array()) {
        Some(arr) => {
            let mut result = Vec::new();
            for (idx, h_value) in arr.iter().enumerate() {
                match parse_single_row_height(h_value, file_path, idx) {
                    Ok(h) => result.push(h),
                    Err(e) => {
                        tracing::warn!(
                            component = "tv.toml.metadata",
                            file_path = %file_path,
                            height_index = idx,
                            error = %e,
                            "Failed to parse row height, skipping"
                        );
                    }
                }
            }
            result
        }
        None => Vec::new(),
    };

    let hidden: Vec<usize> = rows_table
        .get("hidden")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_integer().map(|i| i as usize)).collect())
        .unwrap_or_default();

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        default_height = ?default_height,
        height_count = heights.len(),
        hidden_count = hidden.len(),
        "Parsed row config from metadata"
    );

    Ok(Some(RowConfig { header_height, default_height, frozen_rows, heights, hidden }))
}

/// Parses the metadata.columns section from a TOML file using the real filesystem.
pub fn parse_column_configs_from_file(file_path: &str) -> Result<Vec<ColumnConfig>, TvError> {
    parse_column_configs_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.columns section from a TOML file.
pub fn parse_column_configs_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<ColumnConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_column_configs_from_content(&content, file_path)
}

/// Parses column configurations from TOML content string.
pub fn parse_column_configs_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<ColumnConfig>, TvError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(Vec::new());
    };

    let Some(columns_value) = metadata.get("columns") else {
        return Ok(Vec::new());
    };

    let Some(columns_array) = columns_value.as_array() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.columns is not an array"
        );
        return Ok(Vec::new());
    };

    let mut configs = Vec::new();
    for (idx, col_value) in columns_array.iter().enumerate() {
        match parse_single_column_config(col_value, file_path, idx) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml.metadata",
                    file_path = %file_path,
                    column_index = idx,
                    error = %e,
                    "Failed to parse column config, skipping"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        column_count = configs.len(),
        "Parsed column configs from metadata"
    );

    Ok(configs)
}

fn parse_single_column_config(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<ColumnConfig, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("columns[{idx}] is not a table"),
    })?;

    let key = table
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("columns[{idx}] missing 'key' field"),
        })?
        .to_string();

    let width = table
        .get("width")
        .and_then(|v| v.as_integer())
        .map(|i| i as u32)
        .unwrap_or(100);

    let bold = table.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut config = ColumnConfig::new(key).with_width(width);
    config.bold = bold;
    Ok(config)
}

fn parse_single_row_height(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<RowHeight, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("rows.heights[{idx}] is not a table"),
    })?;

    let row = table
        .get("row")
        .and_then(|v| v.as_integer())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("rows.heights[{idx}] missing 'row' field"),
        })? as usize;

    let height = table
        .get("height")
        .and_then(|v| v.as_integer())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("rows.heights[{idx}] missing 'height' field"),
        })? as u32;

    Ok(RowHeight::new(row, height))
}
