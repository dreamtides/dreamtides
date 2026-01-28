use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::{parse_toml_content, toml_value_to_json};
use crate::toml::metadata_types::{ColumnFilter, FilterCondition, FilterConfig};
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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

    if let Some(val) = table.get("values") {
        let arr = val.as_array().ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("filter.filters[{idx}].condition.values is not an array"),
        })?;
        let values: Vec<serde_json::Value> = arr.iter().map(toml_value_to_json).collect();
        return Ok(FilterCondition::Values(values));
    }

    Err(TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("filter.filters[{idx}].condition has no recognized condition type"),
    })
}
