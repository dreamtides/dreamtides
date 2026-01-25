use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::document_loader::TomlTableData;
use crate::traits::{AtomicWriteError, FileSystem, RealFileSystem};

const TEMP_FILE_PREFIX: &str = ".tv_save_";

/// Represents a single cell update request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellUpdate {
    pub row_index: usize,
    pub column_key: String,
    pub value: serde_json::Value,
}

/// Result of a cell save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCellResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_values: Option<std::collections::HashMap<String, serde_json::Value>>,
}

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

/// Saves a single cell update to the TOML file, preserving document structure.
pub fn save_cell(
    file_path: &str,
    table_name: &str,
    update: &CellUpdate,
) -> Result<SaveCellResult, TvError> {
    save_cell_with_fs(&RealFileSystem, file_path, table_name, update)
}

/// Result of a batch save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBatchResult {
    pub success: bool,
    pub applied_count: usize,
    pub failed_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_updates: Vec<FailedUpdate>,
}

/// Information about a failed cell update within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedUpdate {
    pub row_index: usize,
    pub column_key: String,
    pub reason: String,
}

/// Saves multiple cell updates to the TOML file in a single atomic write.
pub fn save_batch(
    file_path: &str,
    table_name: &str,
    updates: &[CellUpdate],
) -> Result<SaveBatchResult, TvError> {
    save_batch_with_fs(&RealFileSystem, file_path, table_name, updates)
}

/// Saves multiple cell updates using the provided filesystem.
pub fn save_batch_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    updates: &[CellUpdate],
) -> Result<SaveBatchResult, TvError> {
    let start = Instant::now();

    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        cell_count = updates.len(),
        "Starting batch save"
    );

    if updates.is_empty() {
        return Ok(SaveBatchResult {
            success: true,
            applied_count: 0,
            failed_count: 0,
            failed_updates: Vec::new(),
        });
    }

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during batch save"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during batch save"
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
                "Batch save failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let array_len = array.len();
    let mut failed_updates = Vec::new();
    for update in updates {
        if update.row_index >= array_len {
            failed_updates.push(FailedUpdate {
                row_index: update.row_index,
                column_key: update.column_key.clone(),
                reason: format!("Row index {} out of bounds (max: {})", update.row_index, array_len.saturating_sub(1)),
            });
        } else if !update.value.is_null() && json_to_toml_edit_value(&update.value).is_none() {
            failed_updates.push(FailedUpdate {
                row_index: update.row_index,
                column_key: update.column_key.clone(),
                reason: "Unsupported value type".to_string(),
            });
        }
    }

    if !failed_updates.is_empty() {
        tracing::warn!(
            component = "tv.toml",
            file_path = %file_path,
            failed_count = failed_updates.len(),
            "Batch rejected due to validation failure"
        );
        return Ok(SaveBatchResult {
            success: false,
            applied_count: 0,
            failed_count: failed_updates.len(),
            failed_updates,
        });
    }

    for update in updates {
        tracing::debug!(
            component = "tv.toml",
            row_index = update.row_index,
            column_key = %update.column_key,
            "Applying cell update"
        );
        let table = array.get_mut(update.row_index).ok_or_else(|| {
            panic!("Row index {} should be valid after validation", update.row_index)
        })?;
        if let Some(new_value) = json_to_toml_edit_value(&update.value) {
            table[&update.column_key] = new_value;
        } else if update.value.is_null() {
            table.remove(&update.column_key);
        }
    }

    fs.write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        cell_count = updates.len(),
        duration_ms = start.elapsed().as_millis() as u64,
        "Batch saved"
    );

    Ok(SaveBatchResult {
        success: true,
        applied_count: updates.len(),
        failed_count: 0,
        failed_updates: Vec::new(),
    })
}

/// Saves a single cell update using the provided filesystem.
pub fn save_cell_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    update: &CellUpdate,
) -> Result<SaveCellResult, TvError> {
    let start = Instant::now();

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during cell save"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during cell save"
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
                "Cell save failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let table = array.get_mut(update.row_index).ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            row_index = update.row_index,
            error = "Row index out of bounds",
            "Cell save failed"
        );
        TvError::RowNotFound { table_name: table_name.to_string(), row_index: update.row_index }
    })?;

    // Update the cell value
    if let Some(new_value) = json_to_toml_edit_value(&update.value) {
        table[&update.column_key] = new_value;
    } else if update.value.is_null() {
        // Remove the key if the value is null
        table.remove(&update.column_key);
    }

    let output = doc.to_string();

    fs.write_atomic(Path::new(file_path), &output).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        row_index = update.row_index,
        column_key = %update.column_key,
        duration_ms = duration_ms,
        "Cell saved"
    );

    Ok(SaveCellResult { success: true, generated_values: None })
}
