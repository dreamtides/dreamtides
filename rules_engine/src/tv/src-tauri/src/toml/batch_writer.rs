use std::path::Path;
use std::time::Instant;

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::cell_writer::map_atomic_write_error;
use crate::toml::writer_types::{CellUpdate, FailedUpdate, SaveBatchResult};
use crate::toml::{metadata, value_converter};
use crate::traits::{FileSystem, RealFileSystem};
use crate::validation::validation_rules::ValidationRule;
use crate::validation::validators;

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
        None => metadata::parse_validation_rules_from_content(&content, file_path)
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

    let array =
        doc.get_mut(table_name).and_then(|v| v.as_array_of_tables_mut()).ok_or_else(|| {
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
            let results =
                validators::validate_all(&validation_rules, &update.column_key, &update.value);
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

    fs.write_atomic(Path::new(file_path), &doc.to_string())
        .map_err(|e| map_atomic_write_error(e, file_path))?;

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
