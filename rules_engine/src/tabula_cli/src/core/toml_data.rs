use anyhow::{Context, Result};
use toml::Table;
use toml::value::Value;

use crate::core::column_names;
use crate::core::excel_reader::{CellValue, ColumnType, TableInfo};

pub fn canonicalize_numbers(value: Value) -> Value {
    match value {
        Value::Table(table) => Value::Table(
            table.into_iter().map(|(key, value)| (key, canonicalize_numbers(value))).collect(),
        ),
        Value::Array(array) => Value::Array(array.into_iter().map(canonicalize_numbers).collect()),
        Value::String(s) => match s.parse::<i64>() {
            Ok(parsed) if parsed.to_string() == s => Value::Integer(parsed),
            _ => Value::String(s),
        },
        other => other,
    }
}

pub fn table_to_toml(table: &TableInfo) -> Result<String> {
    let table_key = column_names::normalize_table_name(table.name.as_str());
    let ordered_columns: Vec<(String, String)> = table
        .columns
        .iter()
        .filter(|column| matches!(column.column_type, ColumnType::Data))
        .map(|column| {
            (column.name.clone(), column_names::normalize_column_name(column.name.as_str()))
        })
        .collect();

    if ordered_columns.len() == 1 && ordered_columns[0].1 == table_key {
        let (raw_name, _) = &ordered_columns[0];
        let mut values = Vec::new();
        for row in &table.rows {
            if let Some(value) = row.get(raw_name)
                && let Some(toml_value) = cell_value_to_toml(value)
            {
                values.push(toml_value);
            }
        }
        let mut root = Table::new();
        root.insert(table_key.replace('-', "_"), Value::Array(values));
        let root = Value::Table(root);
        let canonical = canonicalize_numbers(root);
        return toml::to_string_pretty(&canonical).context("Failed to serialize TOML");
    }

    let mut rows = Vec::new();
    for row in &table.rows {
        let mut table_row = Table::new();
        for (raw_name, normalized_name) in &ordered_columns {
            if let Some(value) = row.get(raw_name)
                && let Some(toml_value) = cell_value_to_toml(value)
            {
                table_row.insert(normalized_name.clone(), toml_value);
            }
        }
        rows.push(Value::Table(table_row));
    }

    let mut root = Table::new();
    root.insert(table_key, Value::Array(rows));
    let root = Value::Table(root);
    let canonical = canonicalize_numbers(root);
    toml::to_string_pretty(&canonical).context("Failed to serialize TOML")
}

fn cell_value_to_toml(value: &CellValue) -> Option<Value> {
    match value {
        CellValue::Empty => None,
        CellValue::String(s) => Some(Value::String(s.clone())),
        CellValue::Float(f) => Some(Value::Float(*f)),
        CellValue::Int(i) => Some(Value::Integer(*i)),
        CellValue::Bool(b) => Some(Value::Boolean(*b)),
    }
}
