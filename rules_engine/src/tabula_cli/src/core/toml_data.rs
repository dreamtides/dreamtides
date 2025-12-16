use anyhow::{Context, Result};
use toml::Table;
use toml::value::Value;

use crate::core::column_names;
use crate::core::excel_reader::{CellValue, ColumnType, TableInfo};

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
            if let Some(value) = row.get(raw_name) {
                if let Some(toml_value) = cell_value_to_toml(value) {
                    values.push(toml_value);
                }
            }
        }
        let mut root = Table::new();
        root.insert(table_key.replace('-', "_"), Value::Array(values));
        return toml::to_string_pretty(&Value::Table(root)).context("Failed to serialize TOML");
    }

    let mut rows = Vec::new();
    for row in &table.rows {
        let mut table_row = Table::new();
        for (raw_name, normalized_name) in &ordered_columns {
            if let Some(value) = row.get(raw_name) {
                if let Some(toml_value) = cell_value_to_toml(value) {
                    table_row.insert(normalized_name.clone(), toml_value);
                }
            }
        }
        rows.push(Value::Table(table_row));
    }

    let mut root = Table::new();
    root.insert(table_key, Value::Array(rows));
    toml::to_string_pretty(&Value::Table(root)).context("Failed to serialize TOML")
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
