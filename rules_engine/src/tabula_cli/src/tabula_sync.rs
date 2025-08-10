use std::collections::BTreeSet;

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::spreadsheet::{SheetColumn, SheetTable, SheetValue, Spreadsheet};

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
    let mut columns: Vec<SheetColumn> = Vec::new();
    if json_rows.is_empty() {
        let table = SheetTable { name: name.to_string(), columns };
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
            for key in ordered_keys.iter() {
                columns.push(SheetColumn { name: key.clone(), values: Vec::new() });
            }
            for row in json_rows {
                for (idx, key) in ordered_keys.iter().enumerate() {
                    let value = match &row {
                        Value::Object(map) => map.get(key).cloned().unwrap_or(Value::Null),
                        _ => Value::Null,
                    };
                    columns[idx].values.push(SheetValue { data: value });
                }
            }
        }
        _ => {
            let mut col = SheetColumn { name: "value".to_string(), values: Vec::new() };
            for row in json_rows {
                col.values.push(SheetValue { data: row });
            }
            columns.push(col);
        }
    }
    let table = SheetTable { name: name.to_string(), columns };
    sheet.write_table(&table).await?;
    Ok(())
}

pub async fn read_table<T>(sheet: impl Spreadsheet + 'static, name: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let table = sheet.read_table(name).await?;
    if table.columns.is_empty() {
        return Ok(Vec::new());
    }
    if table.columns.len() == 1 && table.columns[0].name == "value" {
        let mut out: Vec<T> = Vec::with_capacity(table.columns[0].values.len());
        for v in table.columns[0].values.iter() {
            let t = serde_json::from_value(v.data.clone())?;
            out.push(t);
        }
        return Ok(out);
    }
    let max_rows = table.columns.iter().map(|c| c.values.len()).max().unwrap_or(0);
    let mut results: Vec<T> = Vec::with_capacity(max_rows);
    for row_idx in 0..max_rows {
        let mut obj = Map::new();
        for col in table.columns.iter() {
            let v = col.values.get(row_idx).map(|sv| sv.data.clone()).unwrap_or(Value::Null);
            obj.insert(col.name.clone(), v);
        }
        let t = serde_json::from_value(Value::Object(obj))?;
        results.push(t);
    }
    Ok(results)
}
