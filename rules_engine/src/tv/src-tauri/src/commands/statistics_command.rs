use crate::error::error_types::TvError;
use crate::toml::metadata;
use crate::toml::metadata_types::StatisticConfig;

#[tauri::command]
pub fn get_statistics_config(file_path: String) -> Result<Vec<StatisticConfig>, TvError> {
    tracing::debug!(
        component = "tv.commands.statistics",
        file_path = %file_path,
        "Loading statistics config"
    );

    let configs = metadata::parse_statistics_from_file(&file_path)?;

    tracing::debug!(
        component = "tv.commands.statistics",
        file_path = %file_path,
        config_count = configs.len(),
        "Statistics config loaded"
    );

    Ok(configs)
}
