use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use toml::map::Map;

#[derive(Serialize, Deserialize)]
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[tauri::command]
pub fn load_toml_table(file_path: String, table_name: String) -> Result<TomlTableData, String> {
    let path = PathBuf::from(&file_path);
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
    let value: toml::Value =
        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))?;
    let table = value
        .get(&table_name)
        .ok_or_else(|| format!("Table '{}' not found in TOML file", table_name))?;
    let array = table
        .as_array()
        .ok_or_else(|| format!("'{}' is not an array of tables", table_name))?;

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

    Ok(TomlTableData { headers, rows })
}

#[tauri::command]
pub fn save_toml_table(
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), String> {
    let mut tables = Vec::new();
    for row in &data.rows {
        let mut table = Map::new();
        for (i, header) in data.headers.iter().enumerate() {
            if let Some(json_val) = row.get(i) {
                if !json_val.is_null() {
                    if let Some(toml_val) = json_to_toml_value(json_val) {
                        table.insert(header.clone(), toml_val);
                    }
                }
            }
        }
        tables.push(toml::Value::Table(table));
    }

    let mut root = Map::new();
    root.insert(table_name, toml::Value::Array(tables));
    let toml_string =
        toml::to_string_pretty(&toml::Value::Table(root)).map_err(|e| format!("{}", e))?;

    let path = PathBuf::from(&file_path);
    fs::write(&path, toml_string)
        .map_err(|e| format!("Failed to write file {}: {}", file_path, e))?;

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

fn json_to_toml_value(value: &serde_json::Value) -> Option<toml::Value> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Some(toml::Value::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Vec<toml::Value> = arr.iter().filter_map(json_to_toml_value).collect();
            Some(toml::Value::Array(toml_arr))
        }
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                if let Some(tv) = json_to_toml_value(v) {
                    map.insert(k.clone(), tv);
                }
            }
            Some(toml::Value::Table(map))
        }
    }
}
