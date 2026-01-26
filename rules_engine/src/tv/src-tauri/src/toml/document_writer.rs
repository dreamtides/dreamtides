use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::document_loader::TomlTableData;
use crate::toml::metadata_parser;
use crate::toml::value_converter;
use crate::traits::{AtomicWriteError, FileSystem, RealFileSystem};
use crate::validation::validation_rules::ValidationRule;
use crate::validation::validators;

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
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let array =
        doc.get_mut(table_name).and_then(|v| v.as_array_of_tables_mut()).ok_or_else(|| {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                error = "Table not found or not an array of tables",
                "Save failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let existing_len = array.len();

    for (row_idx, row) in data.rows.iter().enumerate() {
        if row_idx < existing_len {
            let Some(table) = array.get_mut(row_idx) else {
                break;
            };

            for (col_idx, header) in data.headers.iter().enumerate() {
                if let Some(json_val) = row.get(col_idx) {
                    if json_val.is_null() {
                        table.remove(header);
                    } else if let Some(existing) = table.get_mut(header) {
                        // Use type-preserving conversion to maintain boolean types when the
                        // spreadsheet library returns 0/1 instead of false/true
                        if let Some(new_val) =
                            value_converter::json_to_toml_edit_preserving_type(json_val, existing)
                        {
                            *existing = new_val;
                        }
                    } else if let Some(toml_val) = value_converter::json_to_toml_edit(json_val) {
                        table.insert(header, toml_val);
                    }
                }
            }
        } else {
            let mut new_table = toml_edit::Table::new();
            for (col_idx, header) in data.headers.iter().enumerate() {
                if let Some(json_val) = row.get(col_idx) {
                    if !json_val.is_null() {
                        if let Some(toml_val) = value_converter::json_to_toml_edit(json_val) {
                            new_table.insert(header, toml_val);
                        }
                    }
                }
            }
            array.push(new_table);
            tracing::debug!(
                component = "tv.toml",
                file_path = %file_path,
                row_index = row_idx,
                "New row appended during save"
            );
        }
    }

    // Remove empty rows (all keys cleared) and excess rows (frontend sent fewer
    // rows than exist in TOML, e.g. when the user cleared the last row).
    let mut needs_rebuild = data.rows.len() < existing_len;
    if !needs_rebuild {
        for row_idx in 0..existing_len.min(data.rows.len()) {
            if let Some(table) = array.get(row_idx) {
                if table.is_empty() {
                    needs_rebuild = true;
                    break;
                }
            }
        }
    }

    if needs_rebuild {
        let mut new_array = toml_edit::ArrayOfTables::new();
        for i in 0..array.len() {
            if i >= data.rows.len() && i < existing_len {
                tracing::debug!(
                    component = "tv.toml",
                    file_path = %file_path,
                    row_index = i,
                    "Removing excess row during save"
                );
                continue;
            }
            if let Some(table) = array.get(i) {
                if table.is_empty() {
                    tracing::debug!(
                        component = "tv.toml",
                        file_path = %file_path,
                        row_index = i,
                        "Removing empty row during save"
                    );
                    continue;
                }
                new_array.push(table.clone());
            }
        }
        doc[table_name] = toml_edit::Item::ArrayOfTables(new_array);
    }

    let output = doc.to_string();

    fs.write_atomic(Path::new(file_path), &output)
        .map_err(|e| map_atomic_write_error(e, file_path))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
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
    save_batch_with_fs_and_rules(fs, file_path, table_name, updates, None)
}

/// Saves multiple cell updates with optional validation rules.
pub fn save_batch_with_fs_and_rules(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    updates: &[CellUpdate],
    rules: Option<&[ValidationRule]>,
) -> Result<SaveBatchResult, TvError> {
    let start = Instant::now();

    tracing::debug!(
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

    let validation_rules = match rules {
        Some(r) => r.to_vec(),
        None => metadata_parser::parse_validation_rules_from_content(&content, file_path)
            .unwrap_or_default(),
    };

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during batch save"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
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
                reason: format!(
                    "Row index {} out of bounds (max: {})",
                    update.row_index,
                    array_len.saturating_sub(1)
                ),
            });
        } else if !update.value.is_null()
            && value_converter::json_to_toml_edit(&update.value).is_none()
        {
            failed_updates.push(FailedUpdate {
                row_index: update.row_index,
                column_key: update.column_key.clone(),
                reason: "Unsupported value type".to_string(),
            });
        } else {
            let results = validators::validate_all(&validation_rules, &update.column_key, &update.value);
            if let Some(error) = validators::first_error(&results) {
                tracing::warn!(
                    component = "tv.toml.validation",
                    column = %update.column_key,
                    row = update.row_index,
                    rule_type = %error.rule_type,
                    error = ?error.error_message,
                    "Validation failed in batch"
                );
                failed_updates.push(FailedUpdate {
                    row_index: update.row_index,
                    column_key: update.column_key.clone(),
                    reason: error
                        .error_message
                        .clone()
                        .unwrap_or_else(|| "Validation failed".to_string()),
                });
            }
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
        let table = array.get_mut(update.row_index).unwrap_or_else(|| {
            panic!("Row index {} should be valid after validation", update.row_index)
        });
        if let Some(existing) = table.get(&update.column_key) {
            if let Some(new_value) =
                value_converter::json_to_toml_edit_preserving_type(&update.value, existing)
            {
                table[&update.column_key] = new_value;
            } else if update.value.is_null() {
                table.remove(&update.column_key);
            }
        } else if let Some(new_value) = value_converter::json_to_toml_edit(&update.value) {
            table[&update.column_key] = new_value;
        } else if update.value.is_null() {
            table.remove(&update.column_key);
        }
    }

    fs.write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
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
    save_cell_with_fs_and_rules(fs, file_path, table_name, update, None)
}

/// Saves a single cell update with optional validation rules.
pub fn save_cell_with_fs_and_rules(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    update: &CellUpdate,
    rules: Option<&[ValidationRule]>,
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

    let validation_rules = match rules {
        Some(r) => r.to_vec(),
        None => metadata_parser::parse_validation_rules_from_content(&content, file_path)
            .unwrap_or_default(),
    };

    if let Some(error) =
        validate_cell_value(&validation_rules, &update.column_key, &update.value, update.row_index)
    {
        return Err(error);
    }

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during cell save"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let array =
        doc.get_mut(table_name).and_then(|v| v.as_array_of_tables_mut()).ok_or_else(|| {
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

    if let Some(existing) = table.get(&update.column_key) {
        if let Some(new_value) =
            value_converter::json_to_toml_edit_preserving_type(&update.value, existing)
        {
            table[&update.column_key] = new_value;
        } else if update.value.is_null() {
            table.remove(&update.column_key);
        }
    } else if let Some(new_value) = value_converter::json_to_toml_edit(&update.value) {
        table[&update.column_key] = new_value;
    }

    let output = doc.to_string();

    fs.write_atomic(Path::new(file_path), &output)
        .map_err(|e| map_atomic_write_error(e, file_path))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
        component = "tv.toml",
        file_path = %file_path,
        row_index = update.row_index,
        column_key = %update.column_key,
        duration_ms = duration_ms,
        "Cell saved"
    );

    Ok(SaveCellResult { success: true, generated_values: None })
}

fn validate_cell_value(
    rules: &[ValidationRule],
    column: &str,
    value: &serde_json::Value,
    row_index: usize,
) -> Option<TvError> {
    let results = validators::validate_all(rules, column, value);
    if let Some(error) = validators::first_error(&results) {
        tracing::warn!(
            component = "tv.toml.validation",
            column = %column,
            row = row_index,
            rule_type = %error.rule_type,
            error = ?error.error_message,
            "Validation failed"
        );
        return Some(TvError::ValidationFailed {
            column: column.to_string(),
            row: row_index,
            message: error.error_message.clone().unwrap_or_else(|| "Validation failed".to_string()),
        });
    }
    None
}

/// Result of a row add operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRowResult {
    pub success: bool,
    pub row_index: usize,
}

/// Adds a new row to the TOML array at the specified position.
/// If position is None, appends to the end.
pub fn add_row(
    file_path: &str,
    table_name: &str,
    position: Option<usize>,
    initial_values: Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<AddRowResult, TvError> {
    add_row_with_fs(&RealFileSystem, file_path, table_name, position, initial_values)
}

/// Adds a new row using the provided filesystem.
pub fn add_row_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    position: Option<usize>,
    initial_values: Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<AddRowResult, TvError> {
    let start = Instant::now();

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during add row"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during add row"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let array =
        doc.get_mut(table_name).and_then(|v| v.as_array_of_tables_mut()).ok_or_else(|| {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                error = "Table not found or not an array of tables",
                "Add row failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let array_len = array.len();
    let insert_index = position.unwrap_or(array_len);

    if insert_index > array_len {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            insert_index = insert_index,
            array_len = array_len,
            error = "Insert position out of bounds",
            "Add row failed"
        );
        return Err(TvError::RowNotFound { table_name: table_name.to_string(), row_index: insert_index });
    }

    // Create new table with initial values
    let mut new_table = toml_edit::Table::new();
    if let Some(values) = initial_values {
        for (key, json_val) in values {
            if let Some(toml_val) = value_converter::json_to_toml_edit(&json_val) {
                new_table.insert(&key, toml_val);
            }
        }
    }

    // Insert at position by rebuilding the array
    // toml_edit's ArrayOfTables doesn't have insert(), so we need to rebuild
    let mut new_array = toml_edit::ArrayOfTables::new();
    for i in 0..array_len {
        if i == insert_index {
            new_array.push(new_table.clone());
        }
        if let Some(existing_table) = array.get(i) {
            new_array.push(existing_table.clone());
        }
    }
    // If inserting at end
    if insert_index == array_len {
        new_array.push(new_table);
    }

    doc[table_name] = toml_edit::Item::ArrayOfTables(new_array);

    fs.write_atomic(Path::new(file_path), &doc.to_string())
        .map_err(|e| map_atomic_write_error(e, file_path))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
        component = "tv.toml",
        file_path = %file_path,
        row_index = insert_index,
        duration_ms = duration_ms,
        "Row added"
    );

    Ok(AddRowResult { success: true, row_index: insert_index })
}

/// Result of a row delete operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRowResult {
    pub success: bool,
    pub deleted_index: usize,
}

/// Deletes a row from the TOML array at the specified index.
pub fn delete_row(
    file_path: &str,
    table_name: &str,
    row_index: usize,
) -> Result<DeleteRowResult, TvError> {
    delete_row_with_fs(&RealFileSystem, file_path, table_name, row_index)
}

/// Deletes a row using the provided filesystem.
pub fn delete_row_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    row_index: usize,
) -> Result<DeleteRowResult, TvError> {
    let start = Instant::now();

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during delete row"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during delete row"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let array =
        doc.get_mut(table_name).and_then(|v| v.as_array_of_tables_mut()).ok_or_else(|| {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                error = "Table not found or not an array of tables",
                "Delete row failed"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let array_len = array.len();

    if row_index >= array_len {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            row_index = row_index,
            array_len = array_len,
            error = "Row index out of bounds",
            "Delete row failed"
        );
        return Err(TvError::RowNotFound { table_name: table_name.to_string(), row_index });
    }

    // Rebuild array without the deleted row
    let mut new_array = toml_edit::ArrayOfTables::new();
    for i in 0..array_len {
        if i != row_index {
            if let Some(existing_table) = array.get(i) {
                new_array.push(existing_table.clone());
            }
        }
    }

    doc[table_name] = toml_edit::Item::ArrayOfTables(new_array);

    fs.write_atomic(Path::new(file_path), &doc.to_string())
        .map_err(|e| map_atomic_write_error(e, file_path))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
        component = "tv.toml",
        file_path = %file_path,
        row_index = row_index,
        duration_ms = duration_ms,
        "Row deleted"
    );

    Ok(DeleteRowResult { success: true, deleted_index: row_index })
}
