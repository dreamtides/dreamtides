use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::error_types::TvError;
use crate::toml::value_converter;
use crate::traits::{FileSystem, RealFileSystem};

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Loads a TOML file and extracts the specified table as spreadsheet data.
pub fn load_toml_document(file_path: &str, table_name: &str) -> Result<TomlTableData, TvError> {
    load_toml_document_with_fs(&RealFileSystem, file_path, table_name)
}

/// Loads a TOML file using the provided filesystem implementation.
pub fn load_toml_document_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    table_name: &str,
) -> Result<TomlTableData, TvError> {
    let start = Instant::now();

    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "File not found",
                "Load failed"
            );
            TvError::FileNotFound { path: file_path.to_string() }
        }
        ErrorKind::PermissionDenied => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Permission denied",
                "Load failed"
            );
            TvError::PermissionDenied { path: file_path.to_string(), operation: "read".to_string() }
        }
        ErrorKind::InvalidData => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Invalid UTF-8",
                "Load failed"
            );
            TvError::InvalidUtf8 { path: file_path.to_string(), byte_offset: None }
        }
        _ => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Load failed"
            );
            TvError::FileNotFound { path: file_path.to_string() }
        }
    })?;

    let value: toml::Value = toml::from_str(&content).map_err(|e| {
        let line = e.span().map(|s| content[..s.start].lines().count());
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            line = ?line,
            "TOML parse failed"
        );
        TvError::TomlParseError { path: file_path.to_string(), line, message: e.message().to_string() }
    })?;

    let resolved_name = table_name.replace('-', "_");
    let table = value
        .get(table_name)
        .or_else(|| value.get(&resolved_name))
        .ok_or_else(|| {
            tracing::warn!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                "Skipping file: no matching array-of-tables key found"
            );
            TvError::TableNotFound { table_name: table_name.to_string() }
        })?;

    let array = table.as_array().ok_or_else(|| {
        tracing::warn!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            "Skipping file: value is not an array"
        );
        TvError::NotAnArrayOfTables { table_name: table_name.to_string() }
    })?;

    if !array.is_empty() && !array.iter().any(|item| item.is_table()) {
        tracing::warn!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            "Skipping file: array does not contain tables"
        );
        return Err(TvError::NotAnArrayOfTables { table_name: table_name.to_string() });
    }

    let mut seen = HashSet::new();
    let mut headers = Vec::new();
    for item in array {
        if let Some(tbl) = item.as_table() {
            for key in tbl.keys() {
                if seen.insert(key.clone()) {
                    headers.push(key.clone());
                }
            }
        }
    }
    let mut rows = Vec::new();
    for item in array {
        let mut row = Vec::new();
        if let Some(tbl) = item.as_table() {
            for header in &headers {
                let val =
                    tbl.get(header).map_or(serde_json::Value::Null, value_converter::toml_to_json);
                row.push(val);
            }
        }
        rows.push(row);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::debug!(
        component = "tv.toml",
        file_path = %file_path,
        rows = rows.len(),
        duration_ms = duration_ms,
        "File loaded"
    );

    Ok(TomlTableData { headers, rows })
}
