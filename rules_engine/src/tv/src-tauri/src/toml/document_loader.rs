use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::error_types::TvError;

#[derive(Serialize, Deserialize)]
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Loads a TOML file and extracts the specified table as spreadsheet data.
pub fn load_toml_document(file_path: &str, table_name: &str) -> Result<TomlTableData, TvError> {
    let start = Instant::now();
    let path = PathBuf::from(file_path);

    let content = fs::read_to_string(&path).map_err(|e| match e.kind() {
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
            TvError::PermissionDenied { path: file_path.to_string() }
        }
        ErrorKind::InvalidData => {
            tracing::error!(
                component = "tv.toml",
                file_path = %file_path,
                error = "Invalid UTF-8",
                "Load failed"
            );
            TvError::InvalidUtf8 { path: file_path.to_string() }
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
        TvError::TomlParseError { line, message: e.message().to_string() }
    })?;

    let table = value.get(table_name).ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            error = "Table not found",
            "Load failed"
        );
        TvError::TableNotFound { table_name: table_name.to_string() }
    })?;

    let array = table.as_array().ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            error = "Not an array of tables",
            "Load failed"
        );
        TvError::NotAnArrayOfTables { table_name: table_name.to_string() }
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
                let val = tbl.get(header).map_or(serde_json::Value::Null, toml_value_to_json);
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

pub fn toml_value_to_json(value: &toml::Value) -> serde_json::Value {
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
