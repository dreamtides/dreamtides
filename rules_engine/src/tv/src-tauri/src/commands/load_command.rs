use tauri::AppHandle;

use crate::error::error_types::TvError;
use crate::sync::save_coordinator;
use crate::toml::document_loader::{self, TomlTableData};

/// Tauri command to load a TOML table as spreadsheet data.
#[tauri::command]
pub fn load_toml_table(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
) -> Result<TomlTableData, TvError> {
    save_coordinator::begin_load(&app_handle, &file_path);

    let result = document_loader::load_toml_document(&file_path, &table_name);
    save_coordinator::end_load(&app_handle, &file_path, result.is_ok());

    result
}
