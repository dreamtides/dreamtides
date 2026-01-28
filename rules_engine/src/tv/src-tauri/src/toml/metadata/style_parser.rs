use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::TableStyle;
use crate::traits::{FileSystem, RealFileSystem};

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
    let value = parse_toml_content(content, file_path)?;

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
