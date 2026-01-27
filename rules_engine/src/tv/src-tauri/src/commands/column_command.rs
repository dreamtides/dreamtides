use crate::error::error_types::TvError;
use crate::toml::metadata_types::ColumnConfig;
use crate::toml::{metadata_parser, metadata_serializer};

/// Tauri command to get the column configurations for a TOML file.
#[tauri::command]
pub fn get_column_configs(file_path: String) -> Result<Vec<ColumnConfig>, TvError> {
    tracing::debug!(
        component = "tv.commands.column",
        file_path = %file_path,
        "Loading column configs"
    );

    let configs = metadata_parser::parse_column_configs_from_file(&file_path)?;

    tracing::debug!(
        component = "tv.commands.column",
        file_path = %file_path,
        count = configs.len(),
        "Column configs loaded"
    );

    Ok(configs)
}

/// Tauri command to update a single column's width in the TOML file.
#[tauri::command]
pub fn set_column_width(
    file_path: String,
    column_key: String,
    width: u32,
) -> Result<(), TvError> {
    tracing::info!(
        component = "tv.commands.column",
        file_path = %file_path,
        column_key = %column_key,
        width = width,
        "Setting column width"
    );

    metadata_serializer::update_column_width(&file_path, &column_key, width)?;

    Ok(())
}

/// Tauri command to update a derived column's width in the TOML file.
#[tauri::command]
pub fn set_derived_column_width(
    file_path: String,
    column_name: String,
    width: u32,
) -> Result<(), TvError> {
    tracing::info!(
        component = "tv.commands.column",
        file_path = %file_path,
        column_name = %column_name,
        width = width,
        "Setting derived column width"
    );

    metadata_serializer::update_derived_column_width(&file_path, &column_name, width)?;

    Ok(())
}
