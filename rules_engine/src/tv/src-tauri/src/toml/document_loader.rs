use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::error_types::TvError;
use crate::toml::{array_columns, value_converter};
use crate::traits::TvConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Loads a TOML file and extracts the specified table as spreadsheet data.
pub fn load_toml_document(
    config: &TvConfig,
    file_path: &str,
    table_name: &str,
) -> Result<TomlTableData, TvError> {
    let start = Instant::now();

    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| match e.kind() {
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

    // Phase 1: Discover keys in first-seen order and detect inline scalar arrays.
    let mut seen = HashSet::new();
    let mut raw_keys = Vec::new();
    let mut array_max_len: HashMap<String, usize> = HashMap::new();
    for item in array.iter() {
        if let Some(tbl) = item.as_table() {
            for key in tbl.keys() {
                if seen.insert(key.clone()) {
                    raw_keys.push(key.clone());
                }
                if let Some(toml::Value::Array(arr)) = tbl.get(key) {
                    if is_scalar_array(arr) {
                        let entry = array_max_len.entry(key.clone()).or_insert(0);
                        *entry = (*entry).max(arr.len());
                    }
                }
            }
        }
    }

    // Phase 2: Build expanded headers and rows.
    let mut headers = Vec::new();
    for key in &raw_keys {
        if let Some(&max_len) = array_max_len.get(key) {
            for i in 0..max_len {
                headers.push(array_columns::make_array_column_key(key, i));
            }
        } else {
            headers.push(key.clone());
        }
    }

    let mut rows = Vec::new();
    for item in array {
        let mut row = Vec::new();
        if let Some(tbl) = item.as_table() {
            for key in &raw_keys {
                if let Some(&max_len) = array_max_len.get(key) {
                    let arr = tbl
                        .get(key)
                        .and_then(|v| v.as_array());
                    for i in 0..max_len {
                        let val = arr
                            .and_then(|a| a.get(i))
                            .map_or(serde_json::Value::Null, value_converter::toml_to_json);
                        row.push(val);
                    }
                } else {
                    let val = tbl
                        .get(key)
                        .map_or(serde_json::Value::Null, value_converter::toml_to_json);
                    row.push(val);
                }
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

/// Returns `true` if the array contains only scalar values (strings, integers,
/// floats, booleans), not tables or nested arrays.
fn is_scalar_array(arr: &[toml::Value]) -> bool {
    arr.iter().all(|v| matches!(v, toml::Value::String(_) | toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_)))
}
