use std::io::ErrorKind;
use std::path::Path;

use crate::error::error_types::TvError;
use crate::toml::document_loader::TomlTableData;
use crate::traits::{FileSystem, RealFileSystem};

/// Saves spreadsheet data back to a TOML file, preserving formatting.
pub fn save_toml_document(
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<(), TvError> {
    save_toml_document_with_fs(&RealFileSystem, file_path, table_name, data)
}

/// Saves spreadsheet data back to a TOML file using the provided filesystem.
pub fn save_toml_document_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<(), TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "File not found",
                "Save failed"
            );
            TvError::FileNotFound { path: file_path.to_string() }
        }
        ErrorKind::PermissionDenied => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Permission denied",
                "Save failed"
            );
            TvError::PermissionDenied { path: file_path.to_string() }
        }
        _ => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Save failed"
            );
            TvError::FileNotFound { path: file_path.to_string() }
        }
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

    fs.write(Path::new(file_path), &doc.to_string()).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Write failed"
        );
        TvError::WriteError { path: file_path.to_string(), message: e.to_string() }
    })?;

    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        "File saved"
    );

    Ok(())
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
