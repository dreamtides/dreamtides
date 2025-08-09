use std::collections::HashMap;

use anyhow::Result;
use tabula_cli::spreadsheet::{SheetColumn, SheetTable, SheetValue, Spreadsheet};

#[derive(Default)]
pub struct FakeSpreadsheet {
    sheets: parking_lot::RwLock<HashMap<String, Vec<Vec<String>>>>,
}

impl FakeSpreadsheet {
    pub fn with_sheet(name: &str, rows: Vec<Vec<&str>>) -> Self {
        let mut map = HashMap::new();
        let materialized: Vec<Vec<String>> =
            rows.into_iter().map(|r| r.into_iter().map(|s| s.to_string()).collect()).collect();
        map.insert(name.to_string(), materialized);
        Self { sheets: parking_lot::RwLock::new(map) }
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
            return Ok(SheetTable { name: name.to_string(), columns: vec![] });
        };
        if rows.is_empty() {
            return Ok(SheetTable { name: name.to_string(), columns: vec![] });
        }
        let headers: Vec<String> = rows[0].clone();
        let mut columns: Vec<SheetColumn> =
            headers.into_iter().map(|h| SheetColumn { name: h, values: Vec::new() }).collect();
        for r in rows.iter().skip(1) {
            for (i, col) in columns.iter_mut().enumerate() {
                let v = r.get(i).cloned().unwrap_or_default();
                col.values.push(SheetValue { data: v });
            }
        }
        Ok(SheetTable { name: name.to_string(), columns })
    }

    async fn write_table(&self, table: &SheetTable) -> Result<()> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let header: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
        rows.push(header);
        let max_rows = table.columns.iter().map(|c| c.values.len()).max().unwrap_or(0);
        for i in 0..max_rows {
            let mut r: Vec<String> = Vec::with_capacity(table.columns.len());
            for col in table.columns.iter() {
                let v = col.values.get(i).map(|sv| sv.data.clone()).unwrap_or_default();
                r.push(v);
            }
            rows.push(r);
        }
        let mut guard = self.sheets.write();
        guard.insert(table.name.clone(), rows);
        Ok(())
    }
}
