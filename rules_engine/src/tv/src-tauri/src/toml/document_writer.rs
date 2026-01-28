//! Document-level TOML write operations.
//!
//! This module provides the main entry point for saving spreadsheet data back to TOML files.
//! For more granular operations, see:
//! - [`cell_writer`]: Single cell updates
//! - [`batch_writer`]: Multiple cell updates in a single atomic write
//! - [`row_operations`]: Adding and deleting rows
//! - [`temp_cleanup`]: Orphaned temp file cleanup

use std::path::Path;
use std::time::Instant;

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::cell_writer::map_atomic_write_error;
use crate::toml::document_loader::TomlTableData;
use crate::toml::value_converter;
use crate::traits::TvConfig;
use crate::uuid::uuid_generator;

// Re-export types and functions for backwards compatibility
pub use crate::toml::batch_writer::{save_batch, save_batch_with_rules};
pub use crate::toml::cell_writer::{save_cell, save_cell_with_rules};
pub use crate::toml::row_operations::{add_row, delete_row};
pub use crate::toml::temp_cleanup::cleanup_orphaned_temp_files;
pub use crate::toml::writer_types::{
    AddRowResult, CellUpdate, DeleteRowResult, FailedUpdate, SaveBatchResult, SaveCellResult,
    SaveTableResult,
};

/// Saves spreadsheet data back to a TOML file, preserving formatting.
pub fn save_toml_document(
    config: &TvConfig,
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<SaveTableResult, TvError> {
    let start = Instant::now();

    let read_start = Instant::now();
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Read failed during save"
        );
        map_io_error_for_read(&e, file_path)
    })?;
    let read_duration_ms = read_start.elapsed().as_millis();

    let parse_start = Instant::now();
    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during save"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;
    let parse_duration_ms = parse_start.elapsed().as_millis();

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

    let uuids_generated = doc
        .get_mut(table_name)
        .and_then(|v| v.as_array_of_tables_mut())
        .is_some_and(|array| uuid_generator::ensure_uuids(array, &data.headers));

    let output = doc.to_string();

    let write_start = Instant::now();
    config.fs().write_atomic(Path::new(file_path), &output)
        .map_err(|e| map_atomic_write_error(e, file_path))?;
    let write_duration_ms = write_start.elapsed().as_millis();

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
        component = "tv.toml",
        file_path = %file_path,
        duration_ms = duration_ms,
        read_duration_ms = %read_duration_ms,
        parse_duration_ms = %parse_duration_ms,
        write_duration_ms = %write_duration_ms,
        content_bytes = content.len(),
        output_bytes = output.len(),
        uuids_generated = uuids_generated,
        "File saved"
    );

    Ok(SaveTableResult { uuids_generated })
}
