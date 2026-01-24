use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum TvError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied reading file: {path}")]
    PermissionDenied { path: String },

    #[error("TOML parse error at line {line}: {message}")]
    TomlParseError { line: Option<usize>, message: String },

    #[error("Invalid UTF-8 content in file: {path}")]
    InvalidUtf8 { path: String },

    #[error("Table '{table_name}' not found in TOML file")]
    TableNotFound { table_name: String },

    #[error("'{table_name}' is not an array of tables")]
    NotAnArrayOfTables { table_name: String },

    #[error("Failed to write file {path}: {message}")]
    WriteError { path: String, message: String },
}

impl Serialize for TvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Loads a TOML file and extracts the specified table as spreadsheet data.
#[tauri::command]
pub fn load_toml_table(file_path: String, table_name: String) -> Result<TomlTableData, TvError> {
    let start = Instant::now();
    let path = PathBuf::from(&file_path);

    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "File not found",
                "Load failed"
            );
            TvError::FileNotFound { path: file_path.clone() }
        }
        ErrorKind::PermissionDenied => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Permission denied",
                "Load failed"
            );
            TvError::PermissionDenied { path: file_path.clone() }
        }
        ErrorKind::InvalidData => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Invalid UTF-8",
                "Load failed"
            );
            TvError::InvalidUtf8 { path: file_path.clone() }
        }
        _ => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Load failed"
            );
            TvError::FileNotFound { path: file_path.clone() }
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
        TvError::TomlParseError { line, message: e.message().to_string() }
    })?;

    let table = value.get(&table_name).ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            error = "Table not found",
            "Load failed"
        );
        TvError::TableNotFound { table_name: table_name.clone() }
    })?;

    let array = table.as_array().ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            error = "Not an array of tables",
            "Load failed"
        );
        TvError::NotAnArrayOfTables { table_name: table_name.clone() }
    })?;

    let mut all_keys = BTreeSet::new();
    for item in array {
        if let Some(tbl) = item.as_table() {
            for key in tbl.keys() {
                all_keys.insert(key.clone());
            }
        }
    }

    let headers: Vec<String> = all_keys.into_iter().collect();
    let mut rows = Vec::new();
    for item in array {
        let mut row = Vec::new();
        if let Some(tbl) = item.as_table() {
            for header in &headers {
                let val = tbl
                    .get(header)
                    .map_or(serde_json::Value::Null, toml_value_to_json);
                row.push(val);
            }
        }
        rows.push(row);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        rows = rows.len(),
        duration_ms = duration_ms,
        "File loaded"
    );

    Ok(TomlTableData { headers, rows })
}

/// Saves spreadsheet data back to a TOML file, preserving formatting.
#[tauri::command]
pub fn save_toml_table(
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), TvError> {
    let path = PathBuf::from(&file_path);

    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "File not found",
                "Save failed"
            );
            TvError::FileNotFound { path: file_path.clone() }
        }
        ErrorKind::PermissionDenied => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Permission denied",
                "Save failed"
            );
            TvError::PermissionDenied { path: file_path.clone() }
        }
        _ => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = %e,
                "Save failed"
            );
            TvError::FileNotFound { path: file_path.clone() }
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
        .get_mut(&table_name)
        .and_then(|v| v.as_array_of_tables_mut())
        .ok_or_else(|| {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                table_name = %table_name,
                error = "Table not found or not an array of tables",
                "Save failed"
            );
            TvError::TableNotFound { table_name: table_name.clone() }
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

    fs::write(&path, doc.to_string()).map_err(|e| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            error = %e,
            "Write failed"
        );
        TvError::WriteError { path: file_path.clone(), message: e.to_string() }
    })?;

    tracing::info!(
        component = "tv.toml",
        file_path = %file_path,
        "File saved"
    );

    Ok(())
}

fn toml_value_to_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(tbl) => {
            let map: serde_json::Map<String, serde_json::Value> =
                tbl.iter().map(|(k, v)| (k.clone(), toml_value_to_json(v))).collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
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
