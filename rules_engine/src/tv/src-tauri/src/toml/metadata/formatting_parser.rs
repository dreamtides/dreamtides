use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::{parse_toml_content, toml_value_to_json};
use crate::toml::metadata_types::{ConditionalFormatRule, FormatCondition, FormatStyle};
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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
