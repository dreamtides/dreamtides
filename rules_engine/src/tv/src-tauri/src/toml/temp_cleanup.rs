use std::path::Path;

use crate::error::error_types::TvError;
use crate::traits::TvConfig;

const TEMP_FILE_PREFIX: &str = ".tv_save_";

/// Cleans up orphaned temp files from previous crashes.
pub fn cleanup_orphaned_temp_files(config: &TvConfig, dir_path: &str) -> Result<usize, TvError> {
    let dir = Path::new(dir_path);
    if !config.fs().exists(dir) {
        return Ok(0);
    }

    let temp_files = config.fs().read_dir_temp_files(dir, TEMP_FILE_PREFIX).map_err(|e| {
        tracing::warn!(
            component = "tv.toml",
            dir_path = %dir_path,
            error = %e,
            "Failed to scan for orphaned temp files"
        );
        TvError::WriteError { path: dir_path.to_string(), message: e.to_string() }
    })?;

    let mut removed_count = 0;
    for temp_file in temp_files {
        match config.fs().remove_file(&temp_file) {
            Ok(()) => {
                removed_count += 1;
                tracing::debug!(
                    component = "tv.toml",
                    file_path = %temp_file.display(),
                    "Removed orphaned temp file"
                );
            }
            Err(e) => {
                tracing::warn!(
                    component = "tv.toml",
                    file_path = %temp_file.display(),
                    error = %e,
                    "Failed to remove orphaned temp file"
                );
            }
        }
    }

    if removed_count > 0 {
        tracing::info!(
            component = "tv.toml",
            dir_path = %dir_path,
            removed_count = removed_count,
            "Cleaned up orphaned temp files"
        );
    }

    Ok(removed_count)
}
