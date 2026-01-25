use std::path::Path;
use std::time::Instant;

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::document_loader::TomlTableData;
use crate::traits::{AtomicWriteError, FileSystem, RealFileSystem};

const TEMP_FILE_PREFIX: &str = ".tv_save_";

/// Saves spreadsheet data back to a TOML file, preserving formatting.
pub fn save_toml_document(
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<(), TvError> {
    save_toml_document_with_fs(&RealFileSystem, file_path, table_name, data)
}

/// Cleans up orphaned temp files from previous crashes.
pub fn cleanup_orphaned_temp_files(dir_path: &str) -> Result<usize, TvError> {
    cleanup_orphaned_temp_files_with_fs(&RealFileSystem, dir_path)
}

/// Cleans up orphaned temp files using the provided filesystem.
pub fn cleanup_orphaned_temp_files_with_fs(
    fs: &dyn FileSystem,
    dir_path: &str,
) -> Result<usize, TvError> {
    let dir = Path::new(dir_path);
    if !fs.exists(dir) {
        return Ok(0);
    }

    let temp_files = fs.read_dir_temp_files(dir, TEMP_FILE_PREFIX).map_err(|e| {
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
        match fs.remove_file(&temp_file) {
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

/// Saves spreadsheet data back to a TOML file using the provided filesystem.
pub fn save_toml_document_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<(), TvError> {
    let start = Instant::now();

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during save"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during save"
        );
        TvError::TomlParseError { line: None, message: e.to_string() }
    })?;

    let array = doc
        .get_mut(table_name)
        .and_then(|v| v.as_array_of_tables_mut())
        .ok_or_else(|| {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                error = "Table not found or not an array of tables",
                "Save failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    for (row_idx, row) in data.rows.iter().enumerate() {
        let Some(table) = array.get_mut(row_idx) else {
            break;
        };

        for (col_idx, header) in data.headers.iter().enumerate() {
            if let Some(json_val) = row.get(col_idx) {
                if let Some(existing) = table.get_mut(header) {
                    if let Some(new_val) = json_to_toml_edit_value(json_val) {
                        *existing = new_val;
                    }
                }
            }
        }
    }

    let output = doc.to_string();

    fs.write_atomic(Path::new(file_path), &output).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        duration_ms = duration_ms,
        "File saved"
    );

    Ok(())
}

fn map_atomic_write_error(error: AtomicWriteError, file_path: &str) -> TvError {
    match error {
        AtomicWriteError::TempFileCreate(e) => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Failed to create temp file for atomic write"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Write(e) => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Failed to write content to temp file"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Sync(e) => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Failed to sync temp file"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Rename { source, temp_path } => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                temp_path = %temp_path,
                error = %source,
                "Atomic rename failed"
            );
            TvError::AtomicRenameFailed {
                temp_path,
                target_path: file_path.to_string(),
                message: source.to_string(),
            }
        }
    }
}

fn json_to_toml_edit_value(value: &serde_json::Value) -> Option<toml_edit::Item> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(toml_edit::value(*b)),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(toml_edit::value)
            .or_else(|| n.as_f64().map(toml_edit::value)),
        serde_json::Value::String(s) => Some(toml_edit::value(s.as_str())),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
    }
}
