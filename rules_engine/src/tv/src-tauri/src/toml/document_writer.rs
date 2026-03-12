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
use crate::toml::array_columns;
use crate::toml::cell_writer::map_atomic_write_error;
use crate::toml::document_loader::TomlTableData;
use crate::toml::table_key;
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

    let key = table_key::resolve_key_name(&doc, table_name, file_path, "Save failed")?;
    let array = doc
        .get_mut(&key)
        .and_then(|v| v.as_array_of_tables_mut())
        .ok_or_else(|| TvError::TableNotFound { table_name: table_name.to_string() })?;

    let existing_len = array.len();
    let header_groups = array_columns::group_array_headers(&data.headers);

    for (row_idx, row) in data.rows.iter().enumerate() {
        if row_idx < existing_len {
            let Some(table) = array.get_mut(row_idx) else {
                break;
            };

            // Handle regular (non-array) headers
            for &col_idx in &header_groups.regular_indices {
                let header = &data.headers[col_idx];
                if let Some(json_val) = row.get(col_idx) {
                    if json_val.is_null() {
                        table.remove(header);
                    } else if let Some(existing) = table.get_mut(header) {
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

            // Reassemble array groups
            for group in &header_groups.array_groups {
                write_array_group_to_table(table, group, row);
            }
        } else {
            let mut new_table = toml_edit::Table::new();

            for &col_idx in &header_groups.regular_indices {
                let header = &data.headers[col_idx];
                if let Some(json_val) = row.get(col_idx) {
                    if !json_val.is_null() {
                        if let Some(toml_val) = value_converter::json_to_toml_edit(json_val) {
                            new_table.insert(header, toml_val);
                        }
                    }
                }
            }

            for group in &header_groups.array_groups {
                write_array_group_to_table(&mut new_table, group, row);
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
        doc[&key] = toml_edit::Item::ArrayOfTables(new_array);
    }

    let uuids_generated = doc
        .get_mut(&key)
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

/// Collects non-null values from an array group's columns and writes the
/// reassembled TOML array back into the table row.
fn write_array_group_to_table(
    table: &mut toml_edit::Table,
    group: &array_columns::ArrayGroup,
    row: &[serde_json::Value],
) {
    let values: Vec<&serde_json::Value> = group
        .entries
        .iter()
        .filter_map(|&(col_idx, _)| row.get(col_idx).filter(|v| !v.is_null()))
        .collect();

    if values.is_empty() {
        table.remove(&group.base_key);
    } else {
        let json_arr = serde_json::Value::Array(values.into_iter().cloned().collect());
        if let Some(toml_val) = value_converter::json_to_toml_edit(&json_arr) {
            table.insert(&group.base_key, toml_val);
        }
    }
}
