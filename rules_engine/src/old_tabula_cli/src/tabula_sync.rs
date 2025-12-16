use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use convert_case::Case;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use tabula_data::tabula::TabulaRaw;

use crate::case_utils;
use crate::spreadsheet::{SheetRow, SheetTable, SheetValue, Spreadsheet};

/// Builds the canonical [TabulaRaw] data structure from a list of
/// [SheetTable]s.
pub fn sync(sheets: Vec<SheetTable>) -> Result<TabulaRaw> {
    let mut outer = Map::new();
    for table in sheets {
        let mut rows: Vec<Value> = Vec::with_capacity(table.rows.len());
        for row in table.rows {
            let mut obj = Map::new();
            for (k, v) in row.values {
                obj.insert(case_utils::cleaned_to_case(&k, Case::Snake), v.data);
            }
            rows.push(Value::Object(obj));
        }
        outer.insert(case_utils::cleaned_to_case(&table.name, Case::Snake), Value::Array(rows));
    }
    let raw: TabulaRaw = serde_json::from_value(Value::Object(outer))?;
    Ok(raw)
}

/// Writes a list of structs to a spreadsheet as a table.
pub async fn write_table<T>(
    sheet: impl Spreadsheet + 'static,
    name: &str,
    data: Vec<T>,
) -> Result<()>
where
    T: Serialize,
{
    let mut json_rows: Vec<Value> = Vec::with_capacity(data.len());
    for item in data {
        let v = serde_json::to_value(item)?;
        json_rows.push(v);
    }
    if json_rows.is_empty() {
        let table = SheetTable { name: name.to_string(), rows: Vec::new() };
        sheet.write_table(&table).await?;
        return Ok(());
    }
    match &json_rows[0] {
        Value::Object(_) => {
            let mut keys: BTreeSet<String> = BTreeSet::new();
            for row in json_rows.iter() {
                if let Value::Object(map) = row {
                    for k in map.keys() {
                        keys.insert(k.clone());
                    }
                }
            }
            let ordered_keys: Vec<String> = keys.into_iter().collect();
            let mut rows: Vec<SheetRow> = Vec::with_capacity(json_rows.len());
            for row in json_rows {
                let mut map: BTreeMap<String, SheetValue> = BTreeMap::new();
                for key in ordered_keys.iter() {
                    let value = match &row {
                        Value::Object(m) => m.get(key).cloned().unwrap_or(Value::Null),
                        _ => Value::Null,
                    };
                    map.insert(key.clone(), SheetValue { data: value });
                }
                rows.push(SheetRow { values: map });
            }
            let table = SheetTable { name: name.to_string(), rows };
            sheet.write_table(&table).await?;
            Ok(())
        }
        _ => {
            let mut rows: Vec<SheetRow> = Vec::with_capacity(json_rows.len());
            for row in json_rows {
                let mut map: BTreeMap<String, SheetValue> = BTreeMap::new();
                map.insert("value".to_string(), SheetValue { data: row });
                rows.push(SheetRow { values: map });
            }
            let table = SheetTable { name: name.to_string(), rows };
            sheet.write_table(&table).await?;
            Ok(())
        }
    }
}

pub async fn read_table<T>(sheet: impl Spreadsheet + 'static, name: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let table = sheet.read_table(name).await?;
    if table.rows.is_empty() {
        return Ok(Vec::new());
    }
    let mut results: Vec<T> = Vec::with_capacity(table.rows.len());
    for row in table.rows {
        let mut obj = Map::new();
        for (k, v) in row.values {
            obj.insert(k, v.data);
        }
        let t = serde_json::from_value(Value::Object(obj))?;
        results.push(t);
    }
    Ok(results)
}
