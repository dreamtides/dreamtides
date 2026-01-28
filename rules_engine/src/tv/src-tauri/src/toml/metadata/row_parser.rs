use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::{RowConfig, RowHeight};
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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
