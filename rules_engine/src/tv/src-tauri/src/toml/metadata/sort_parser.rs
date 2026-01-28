use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::SortConfig;
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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
