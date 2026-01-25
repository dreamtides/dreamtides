use crate::error::error_types::TvError;
use crate::toml::document_loader::TomlTableData;
use crate::toml::document_writer::{self, CellUpdate, SaveCellResult};

/// Tauri command to save spreadsheet data back to a TOML file.
#[tauri::command]
pub fn save_toml_table(
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), TvError> {
    document_writer::save_toml_document(&file_path, &table_name, &data)
}

/// Tauri command to save a single cell update to a TOML file.
#[tauri::command]
pub fn save_cell(
    file_path: String,
    table_name: String,
    row_index: usize,
    column_key: String,
    value: serde_json::Value,
) -> Result<SaveCellResult, TvError> {
    let update = CellUpdate { row_index, column_key, value };
    document_writer::save_cell(&file_path, &table_name, &update)
}
