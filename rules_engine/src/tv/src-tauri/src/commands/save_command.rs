use tauri::AppHandle;

use crate::error::error_types::TvError;
use crate::sync::save_coordinator;
use crate::toml::document_loader::TomlTableData;
use crate::toml::document_writer::{self, CellUpdate, SaveBatchResult, SaveCellResult};

/// Tauri command to save spreadsheet data back to a TOML file.
#[tauri::command]
pub fn save_toml_table(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), TvError> {
    save_coordinator::begin_save(&app_handle, &file_path)?;

    let result = document_writer::save_toml_document(&file_path, &table_name, &data);
    let _ = save_coordinator::end_save(&app_handle, &file_path, result.is_ok());

    result
}

/// Tauri command to save a single cell update to a TOML file.
#[tauri::command]
pub fn save_cell(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    row_index: usize,
    column_key: String,
    value: serde_json::Value,
) -> Result<SaveCellResult, TvError> {
    save_coordinator::begin_save(&app_handle, &file_path)?;

    let update = CellUpdate { row_index, column_key, value };
    let result = document_writer::save_cell(&file_path, &table_name, &update);
    let _ = save_coordinator::end_save(&app_handle, &file_path, result.is_ok());

    result
}

/// Tauri command to save multiple cell updates in a single atomic write.
#[tauri::command]
pub fn save_batch(
    file_path: String,
    table_name: String,
    updates: Vec<CellUpdate>,
) -> Result<SaveBatchResult, TvError> {
    document_writer::save_batch(&file_path, &table_name, &updates)
}
