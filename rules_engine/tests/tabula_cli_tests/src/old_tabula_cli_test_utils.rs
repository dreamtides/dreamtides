use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use old_tabula_cli::spreadsheet::{SheetRow, SheetTable, SheetValue, Spreadsheet};
use parking_lot::RwLock;
use serde_json::Value;

#[derive(Default)]
pub struct FakeSpreadsheet {
    sheets: RwLock<HashMap<String, Vec<Vec<String>>>>,
}

impl FakeSpreadsheet {
    pub fn with_sheet(name: &str, rows: Vec<Vec<&str>>) -> Self {
        let mut map = HashMap::new();
        let materialized: Vec<Vec<String>> =
            rows.into_iter().map(|r| r.into_iter().map(|s| s.to_string()).collect()).collect();
        map.insert(name.to_string(), materialized);
        Self { sheets: RwLock::new(map) }
    }

    fn col_to_index(col: &str) -> usize {
        let mut n: usize = 0;
        for ch in col.chars() {
            let v = (ch as u8 - b'A') as usize + 1;
            n = n * 26 + v;
        }
        n - 1
    }
}

impl Spreadsheet for FakeSpreadsheet {
    async fn read_cell(&self, sheet: &str, column: &str, row: u32) -> Result<Option<String>> {
        let col_idx = Self::col_to_index(column);
        let row_idx = if row == 0 { 0 } else { (row - 1) as usize };
        let guard = self.sheets.read();
        let Some(rows) = guard.get(sheet) else { return Ok(None) };
        let Some(r) = rows.get(row_idx) else { return Ok(None) };
        Ok(r.get(col_idx).cloned())
    }

    async fn write_cell(&self, sheet: &str, column: &str, row: u32, value: &str) -> Result<()> {
        let col_idx = Self::col_to_index(column);
        let row_idx = if row == 0 { 0 } else { (row - 1) as usize };
        let mut guard = self.sheets.write();
        let rows = guard.entry(sheet.to_string()).or_default();
        if rows.len() <= row_idx {
            rows.resize(row_idx + 1, Vec::new());
        }
        let r = &mut rows[row_idx];
        if r.len() <= col_idx {
            r.resize(col_idx + 1, String::new());
        }
        r[col_idx] = value.to_string();
        Ok(())
    }

    async fn read_table(&self, name: &str) -> Result<SheetTable> {
        let guard = self.sheets.read();
        let Some(rows) = guard.get(name) else {
            return Ok(SheetTable { name: name.to_string(), rows: vec![] });
        };
        if rows.is_empty() {
            return Ok(SheetTable { name: name.to_string(), rows: vec![] });
        }
        let headers: Vec<String> = rows[0].clone();
        let mut out_rows: Vec<SheetRow> = Vec::new();
        for r in rows.iter().skip(1) {
            let mut map: BTreeMap<String, SheetValue> = BTreeMap::new();
            for (i, key) in headers.iter().enumerate() {
                let v = r.get(i).cloned().unwrap_or_default();
                map.insert(key.clone(), SheetValue { data: Value::String(v) });
            }
            out_rows.push(SheetRow { values: map });
        }
        Ok(SheetTable { name: name.to_string(), rows: out_rows })
    }

    async fn write_table(&self, table: &SheetTable) -> Result<()> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut keys: Vec<String> =
            table.rows.iter().flat_map(|r| r.values.keys().cloned()).collect();
        keys.sort();
        keys.dedup();
        let header: Vec<String> = if keys.is_empty() { vec![String::new()] } else { keys.clone() };
        rows.push(header);
        for row in table.rows.iter() {
            let mut r: Vec<String> = Vec::with_capacity(keys.len().max(1));
            if keys.is_empty() {
                r.push(String::new());
            } else {
                for k in keys.iter() {
                    let v = row
                        .values
                        .get(k)
                        .map(|sv| match &sv.data {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => {
                                if *b {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                }
                            }
                            Value::Null => String::new(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();
                    r.push(v);
                }
            }
            rows.push(r);
        }
        let mut guard = self.sheets.write();
        guard.insert(table.name.clone(), rows);
        Ok(())
    }

    async fn read_all_tables(&self) -> Result<Vec<SheetTable>> {
        let guard = self.sheets.read();
        let mut out: Vec<SheetTable> = Vec::with_capacity(guard.len());
        let mut names: Vec<String> = guard.keys().cloned().collect();
        names.sort();
        for name in names {
            let Some(rows) = guard.get(&name) else { continue };
            if rows.is_empty() {
                out.push(SheetTable { name: name.clone(), rows: vec![] });
                continue;
            }
            let headers: Vec<String> = rows[0].clone();
            let mut out_rows: Vec<SheetRow> = Vec::new();
            for r in rows.iter().skip(1) {
                let mut map: BTreeMap<String, SheetValue> = BTreeMap::new();
                for (i, key) in headers.iter().enumerate() {
                    let v = r.get(i).cloned().unwrap_or_default();
                    map.insert(key.clone(), SheetValue { data: Value::String(v) });
                }
                out_rows.push(SheetRow { values: map });
            }
            out.push(SheetTable { name: name.clone(), rows: out_rows });
        }
        Ok(out)
    }
}
