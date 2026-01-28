use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::DerivedColumnConfig;
use crate::traits::{FileSystem, RealFileSystem};

/// Parses the metadata.derived_columns section from a TOML file using the real filesystem.
pub fn parse_derived_columns_from_file(file_path: &str) -> Result<Vec<DerivedColumnConfig>, TvError> {
    parse_derived_columns_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.derived_columns section from a TOML file.
pub fn parse_derived_columns_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<DerivedColumnConfig>, TvError> {
    let read_start = std::time::Instant::now();
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;
    let read_duration_ms = read_start.elapsed().as_millis();

    let parse_start = std::time::Instant::now();
    let result = parse_derived_columns_from_content(&content, file_path);
    let parse_duration_ms = parse_start.elapsed().as_millis();

    tracing::info!(
        component = "tv.toml.metadata.derived",
        file_path = %file_path,
        content_bytes = content.len(),
        read_duration_ms = %read_duration_ms,
        parse_duration_ms = %parse_duration_ms,
        "Parsed derived columns from file"
    );

    result
}

/// Parses derived column configurations from TOML content string.
pub fn parse_derived_columns_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<DerivedColumnConfig>, TvError> {
    let value = parse_toml_content(content, file_path)?;

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

    let frozen = table.get("frozen").and_then(|v| v.as_bool()).unwrap_or(false);

    let url_template = table.get("url_template").and_then(|v| v.as_str()).map(String::from);

    Ok(DerivedColumnConfig { name, function, position, width, frozen, inputs, url_template })
}
