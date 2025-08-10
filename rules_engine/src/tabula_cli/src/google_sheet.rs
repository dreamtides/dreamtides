use anyhow::Result;
use google_sheets4::Sheets;
use google_sheets4::api::ValueRange;
use hyper_util::client::legacy::connect::HttpConnector;
use serde_json::Value;
use yup_oauth2::hyper_rustls::HttpsConnector;

use crate::spreadsheet::{SheetColumn, SheetTable, SheetValue, Spreadsheet};

pub struct GoogleSheet {
    sheet_id: String,
    hub: Sheets<HttpsConnector<HttpConnector>>,
}

impl GoogleSheet {
    pub fn new(sheet_id: String, hub: Sheets<HttpsConnector<HttpConnector>>) -> Self {
        Self { sheet_id, hub }
    }
}

impl Spreadsheet for GoogleSheet {
    async fn read_cell(&self, sheet: &str, column: &str, row: u32) -> Result<Option<String>> {
        let sheet = sheet.to_string();
        let column = column.to_string();
        let range = format!("{sheet}!{column}{row}");
        let (_, value_range) = self
            .hub
            .spreadsheets()
            .values_get(&self.sheet_id, &range)
            .add_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
            .doit()
            .await?;
        let values = value_range.values.unwrap_or_default();
        let Some(first_row) = values.first() else { return Ok(None) };
        let Some(first_cell) = first_row.first() else { return Ok(None) };
        let s = match first_cell {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            Value::Null => String::new(),
            other => other.to_string(),
        };
        Ok(Some(s))
    }

    async fn write_cell(&self, sheet: &str, column: &str, row: u32, value: &str) -> Result<()> {
        let sheet = sheet.to_string();
        let column = column.to_string();
        let value = value.to_string();
        let range = format!("{sheet}!{column}{row}");
        let value_range =
            ValueRange { values: Some(vec![vec![Value::String(value)]]), ..Default::default() };
        let _ = self
            .hub
            .spreadsheets()
            .values_update(value_range, &self.sheet_id, &range)
            .value_input_option("RAW")
            .add_scope("https://www.googleapis.com/auth/spreadsheets")
            .doit()
            .await?;
        Ok(())
    }

    async fn read_table(&self, name: &str) -> Result<SheetTable> {
        let name = name.to_string();
        let result = self
            .hub
            .spreadsheets()
            .values_get(&self.sheet_id, &name)
            .add_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
            .doit()
            .await?;
        let value_range = result.1;
        let values = value_range.values.unwrap_or_default();
        if values.is_empty() {
            return Ok(SheetTable { name, columns: Vec::new() });
        }
        let header_cells = values.first().cloned().unwrap_or_default();
        let headers: Vec<String> = header_cells
            .into_iter()
            .map(|v| match v {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => {
                    if b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Value::Null => String::new(),
                other => other.to_string(),
            })
            .collect();
        if headers.is_empty() {
            return Ok(SheetTable { name, columns: Vec::new() });
        }
        let num_columns = headers.len();
        let mut columns: Vec<SheetColumn> =
            headers.into_iter().map(|h| SheetColumn { name: h, values: Vec::new() }).collect();
        for row in values.into_iter().skip(1) {
            for (col_idx, column) in columns.iter_mut().enumerate().take(num_columns) {
                let cell = row.get(col_idx).cloned().unwrap_or(Value::Null);
                column.values.push(SheetValue { data: cell });
            }
        }
        Ok(SheetTable { name, columns })
    }

    async fn write_table(&self, table: &SheetTable) -> Result<()> {
        let name = table.name.clone();
        let columns = &table.columns;
        let header: Vec<Value> = columns.iter().map(|c| Value::String(c.name.clone())).collect();
        let max_rows = columns.iter().map(|c| c.values.len()).max().unwrap_or(0);
        let mut rows: Vec<Vec<Value>> = Vec::with_capacity(max_rows + 1);
        rows.push(header);
        for row_idx in 0..max_rows {
            let mut row: Vec<Value> = Vec::with_capacity(columns.len());
            for col in columns.iter() {
                let cell = col.values.get(row_idx).map(|v| v.data.clone()).unwrap_or(Value::Null);
                row.push(cell);
            }
            rows.push(row);
        }
        let end_col =
            number_to_column_letters(if columns.is_empty() { 1 } else { columns.len() as u32 });
        let end_row = if rows.is_empty() { 1 } else { rows.len() as u32 };
        let range = format!("{name}!A1:{end_col}{end_row}");
        let value_range = ValueRange { values: Some(rows), ..Default::default() };
        let _ = self
            .hub
            .spreadsheets()
            .values_update(value_range, &self.sheet_id, &range)
            .value_input_option("RAW")
            .add_scope("https://www.googleapis.com/auth/spreadsheets")
            .doit()
            .await?;
        Ok(())
    }
}

fn number_to_column_letters(mut number: u32) -> String {
    let mut letters = String::new();
    if number == 0 {
        return "A".to_string();
    }
    while number > 0 {
        let mut rem = number % 26;
        if rem == 0 {
            rem = 26;
        }
        let ch = ((rem - 1) as u8 + b'A') as char;
        letters.insert(0, ch);
        let mut reduced = number - rem;
        let mut quotient = 0;
        while reduced >= 26 {
            reduced -= 26;
            quotient += 1;
        }
        number = quotient;
    }
    letters
}
