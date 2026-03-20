use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::metadata::helpers::parse_toml_content;
use crate::toml::metadata_types::StatisticConfig;
use crate::traits::{FileSystem, RealFileSystem};

/// Parses the metadata.statistics section from a TOML file using the real filesystem.
pub fn parse_statistics_from_file(file_path: &str) -> Result<Vec<StatisticConfig>, TvError> {
    parse_statistics_with_fs(&RealFileSystem, file_path)
}

/// Parses the metadata.statistics section from a TOML file.
pub fn parse_statistics_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
) -> Result<Vec<StatisticConfig>, TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|_| {
        TvError::FileNotFound { path: file_path.to_string() }
    })?;

    parse_statistics_from_content(&content, file_path)
}

/// Parses statistics configurations from TOML content string.
pub fn parse_statistics_from_content(
    content: &str,
    file_path: &str,
) -> Result<Vec<StatisticConfig>, TvError> {
    let value = parse_toml_content(content, file_path)?;

    let Some(metadata) = value.get("metadata") else {
        return Ok(Vec::new());
    };

    let Some(statistics_value) = metadata.get("statistics") else {
        return Ok(Vec::new());
    };

    let Some(statistics_array) = statistics_value.as_array() else {
        tracing::warn!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            "metadata.statistics is not an array"
        );
        return Ok(Vec::new());
    };

    let mut configs = Vec::new();
    for (idx, stat_value) in statistics_array.iter().enumerate() {
        match parse_single_statistic_config(stat_value, file_path, idx) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml.metadata",
                    file_path = %file_path,
                    statistic_index = idx,
                    error = %e,
                    "Failed to parse statistic config, skipping"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        statistic_count = configs.len(),
        "Parsed statistics configs from metadata"
    );

    Ok(configs)
}

fn parse_single_statistic_config(
    value: &toml::Value,
    file_path: &str,
    idx: usize,
) -> Result<StatisticConfig, TvError> {
    let table = value.as_table().ok_or_else(|| TvError::MetadataCorrupt {
        path: file_path.to_string(),
        message: format!("statistics[{idx}] is not a table"),
    })?;

    let column = table
        .get("column")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("statistics[{idx}] missing 'column' field"),
        })?
        .to_string();

    let label = table
        .get("label")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TvError::MetadataCorrupt {
            path: file_path.to_string(),
            message: format!("statistics[{idx}] missing 'label' field"),
        })?
        .to_string();

    let statistic_type = table
        .get("statistic_type")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "value_counts" => crate::toml::metadata_types::StatisticType::ValueCounts,
            _ => crate::toml::metadata_types::StatisticType::ValueCounts,
        })
        .unwrap_or_default();

    Ok(StatisticConfig { column, label, statistic_type })
}
