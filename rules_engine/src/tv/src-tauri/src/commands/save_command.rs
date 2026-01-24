use crate::error::error_types::TvError;
use crate::toml::document_loader::TomlTableData;
use crate::toml::document_writer;

/// Tauri command to save spreadsheet data back to a TOML file.
#[tauri::command]
pub fn save_toml_table(
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), TvError> {
    document_writer::save_toml_document(&file_path, &table_name, &data)
}
