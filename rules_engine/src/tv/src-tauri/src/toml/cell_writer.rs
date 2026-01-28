use std::path::Path;
use std::time::Instant;

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::{metadata, value_converter};
use crate::toml::writer_types::{CellUpdate, SaveCellResult};
use crate::traits::{AtomicWriteError, FileSystem, RealFileSystem};
use crate::validation::validation_rules::ValidationRule;
use crate::validation::validators;

/// Saves a single cell update to the TOML file, preserving document structure.
pub fn save_cell(
    file_path: &str,
    table_name: &str,
    update: &CellUpdate,
) -> Result<SaveCellResult, TvError> {
    save_cell_with_fs(&RealFileSystem, file_path, table_name, update)
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
        None => metadata::parse_validation_rules_from_content(&content, file_path)
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

pub(crate) fn map_atomic_write_error(error: AtomicWriteError, file_path: &str) -> TvError {
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
