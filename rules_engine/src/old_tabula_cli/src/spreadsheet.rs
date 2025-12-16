use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::Value;

pub trait Spreadsheet {
    fn read_cell(
        &self,
        sheet: &str,
        column: &str,
        row: u32,
    ) -> impl Future<Output = Result<Option<String>>> + Send;

    fn write_cell(
        &self,
        sheet: &str,
        column: &str,
        row: u32,
        value: &str,
    ) -> impl Future<Output = Result<()>> + Send;

    fn read_table(&self, name: &str) -> impl Future<Output = Result<SheetTable>> + Send;

    fn write_table(&self, table: &SheetTable) -> impl Future<Output = Result<()>> + Send;

    fn read_all_tables(&self) -> impl Future<Output = Result<Vec<SheetTable>>> + Send;
}

pub struct SheetTable {
    pub name: String,
    pub rows: Vec<SheetRow>,
}

/// Represents a single row in a sheet.
pub struct SheetRow {
    pub values: BTreeMap<String, SheetValue>,
}

/// Represents a single value in a column.
pub struct SheetValue {
    pub data: Value,
}
