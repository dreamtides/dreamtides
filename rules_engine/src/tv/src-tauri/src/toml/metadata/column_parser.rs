use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::ColumnConfig;
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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
    let frozen = table.get("frozen").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut config = ColumnConfig::new(key).with_width(width);
    config.bold = bold;
    config.frozen = frozen;
    Ok(config)
}
