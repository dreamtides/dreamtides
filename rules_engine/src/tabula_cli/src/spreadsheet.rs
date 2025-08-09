use anyhow::Result;

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
}

/// Represents a single sheet in a spreadsheet.
pub struct SheetTable {
    pub name: String,
    pub columns: Vec<SheetColumn>,
}

/// Represents a single column in a sheet.
pub struct SheetColumn {
    pub name: String,
    pub values: Vec<SheetValue>,
}

/// Represents a single value in a column.
pub struct SheetValue {
    pub data: String,
}
