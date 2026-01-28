use std::path::Path;
use std::time::Instant;

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::cell_writer::map_atomic_write_error;
use crate::toml::value_converter;
use crate::toml::writer_types::{AddRowResult, DeleteRowResult};
use crate::traits::{FileSystem, RealFileSystem};

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
        return Err(TvError::RowNotFound {
            table_name: table_name.to_string(),
            row_index: insert_index,
        });
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
