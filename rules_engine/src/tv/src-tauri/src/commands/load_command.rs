use crate::error::error_types::TvError;
use crate::toml::document_loader::{self, TomlTableData};

/// Tauri command to load a TOML table as spreadsheet data.
#[tauri::command]
pub fn load_toml_table(file_path: String, table_name: String) -> Result<TomlTableData, TvError> {
    document_loader::load_toml_document(&file_path, &table_name)
}
