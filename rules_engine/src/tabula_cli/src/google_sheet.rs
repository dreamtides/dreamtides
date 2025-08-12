use anyhow::Result;
use google_sheets4::Sheets;
use google_sheets4::api::{Spreadsheet as GSpreadsheet, ValueRange};
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
        let (_, value_range) = self
            .hub
            .spreadsheets()
            .values_get(&self.sheet_id, &name)
            .add_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
            .doit()
            .await?;
        let values = value_range.values.unwrap_or_default();
        Ok(values_to_table(name, values))
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

    async fn read_all_tables(&self) -> Result<Vec<SheetTable>> {
        let (_, spreadsheet): (_, GSpreadsheet) = self
            .hub
            .spreadsheets()
            .get(&self.sheet_id)
            .include_grid_data(true)
            .add_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
            .doit()
            .await?;

        let mut tables: Vec<SheetTable> = Vec::new();
        let sheets = spreadsheet.sheets.unwrap_or_default();
        for sheet in sheets {
            let name = sheet.properties.and_then(|p| p.title).unwrap_or_default();

            let grid_sets = sheet.data.unwrap_or_default();
            let mut all_rows: Vec<google_sheets4::api::RowData> = Vec::new();
            for grid in grid_sets {
                let mut rows = grid.row_data.unwrap_or_default();
                if !rows.is_empty() {
                    all_rows.append(&mut rows);
                }
            }

            if all_rows.is_empty() {
                tables.push(SheetTable { name, columns: Vec::new() });
                continue;
            }

            let header_cells = all_rows[0].values.clone().unwrap_or_default();
            let header_row: Vec<Value> = header_cells
                .into_iter()
                .map(|cell| {
                    if let Some(ev) = cell.effective_value {
                        if let Some(s) = ev.string_value {
                            Value::String(s)
                        } else if let Some(n) = ev.number_value {
                            Value::String(n.to_string())
                        } else if let Some(b) = ev.bool_value {
                            if b {
                                Value::String("true".to_string())
                            } else {
                                Value::String("false".to_string())
                            }
                        } else {
                            Value::String(ev.formula_value.unwrap_or_default())
                        }
                    } else {
                        Value::String(cell.formatted_value.unwrap_or_default())
                    }
                })
                .collect();

            if header_row.is_empty() {
                tables.push(SheetTable { name, columns: Vec::new() });
                continue;
            }

            let mut values: Vec<Vec<Value>> = Vec::new();
            values.push(header_row);
            for row in all_rows.into_iter().skip(1) {
                let cells = row.values.unwrap_or_default();
                let row_vals: Vec<Value> = cells
                    .into_iter()
                    .map(|c| {
                        if let Some(ev) = c.effective_value {
                            if let Some(s) = ev.string_value {
                                Value::String(s)
                            } else if let Some(n) = ev.number_value {
                                match serde_json::Number::from_f64(n) {
                                    Some(num) => Value::Number(num),
                                    None => Value::Null,
                                }
                            } else if let Some(b) = ev.bool_value {
                                Value::Bool(b)
                            } else {
                                Value::Null
                            }
                        } else if let Some(fv) = c.formatted_value {
                            Value::String(fv)
                        } else {
                            Value::Null
                        }
                    })
                    .collect();
                values.push(row_vals);
            }

            tables.push(values_to_table(name, values));
        }

        Ok(tables)
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

fn values_to_table(name: String, values: Vec<Vec<Value>>) -> SheetTable {
    if values.is_empty() {
        return SheetTable { name, columns: Vec::new() };
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
        return SheetTable { name, columns: Vec::new() };
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
    SheetTable { name, columns }
}
