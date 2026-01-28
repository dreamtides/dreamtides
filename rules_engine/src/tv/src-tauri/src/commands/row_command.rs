use crate::error::error_types::TvError;
use crate::toml::metadata;
use crate::toml::metadata_types::RowConfig;

/// Tauri command to get the row configuration for a TOML file.
#[tauri::command]
pub fn get_row_config(file_path: String) -> Result<Option<RowConfig>, TvError> {
    tracing::debug!(
        component = "tv.commands.row",
        file_path = %file_path,
        "Loading row config"
    );

    let row_config = metadata::parse_row_config_from_file(&file_path)?;

    tracing::debug!(
        component = "tv.commands.row",
        file_path = %file_path,
        has_config = row_config.is_some(),
        "Row config loaded"
    );

    Ok(row_config)
}
