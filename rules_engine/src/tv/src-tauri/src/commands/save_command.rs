use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::ability_parser::ability_parser_state::AbilityParserState;
use crate::error::error_types::TvError;
use crate::error::permission_recovery::{self, PermissionState};
use crate::sync::state_machine;
use crate::toml::document_loader::TomlTableData;
use crate::toml::document_writer::{
    self, AddRowResult, CellUpdate, SaveBatchResult, SaveCellResult, SaveTableResult,
};
use crate::traits::TvConfig;

/// Tauri command to save spreadsheet data back to a TOML file.
#[tauri::command]
pub fn save_toml_table(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<SaveTableResult, TvError> {
    state_machine::begin_save(&app_handle, &file_path)?;

    let result = document_writer::save_toml_document(&TvConfig::default(), &file_path, &table_name, &data);
    let _ = state_machine::end_save(&app_handle, &file_path, result.is_ok());

    if result.is_ok() {
        let column_keys: Vec<&str> = data.headers.iter().map(|s| s.as_str()).collect();
        trigger_ability_parse_if_needed(&app_handle, &file_path, &column_keys);
    }

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
    // Check if file is in read-only mode due to permission issues
    let permission_state = permission_recovery::get_permission_state(&app_handle, &file_path);
    if permission_state == PermissionState::ReadOnly {
        // Queue the update for later retry
        let update = CellUpdate { row_index, column_key: column_key.clone(), value };
        permission_recovery::queue_pending_update(&app_handle, &file_path, &table_name, update);

        tracing::warn!(
            component = "tv.commands.save",
            file_path = %file_path,
            row_index = row_index,
            column_key = %column_key,
            "Save rejected due to read-only permissions, update queued"
        );

        return Err(TvError::PermissionDenied {
            path: file_path,
            operation: "write".to_string(),
        });
    }

    state_machine::begin_save(&app_handle, &file_path)?;

    let update = CellUpdate { row_index, column_key: column_key.clone(), value: value.clone() };
    let result = document_writer::save_cell(&TvConfig::default(), &file_path, &table_name, &update);

    // Handle permission errors by updating state and queueing the update
    if let Err(ref e) = result {
        if matches!(e, TvError::PermissionDenied { .. }) {
            let new_state = permission_recovery::handle_permission_error(&app_handle, &file_path, e);
            if new_state == PermissionState::ReadOnly {
                let update_to_queue =
                    CellUpdate { row_index, column_key: column_key.clone(), value };
                permission_recovery::queue_pending_update(
                    &app_handle,
                    &file_path,
                    &table_name,
                    update_to_queue,
                );
            }
        }
    }

    let _ = state_machine::end_save(&app_handle, &file_path, result.is_ok());

    if result.is_ok() {
        trigger_ability_parse_if_needed(&app_handle, &file_path, &[&column_key]);
    }

    result
}

/// Tauri command to save multiple cell updates in a single atomic write.
#[tauri::command]
pub fn save_batch(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    updates: Vec<CellUpdate>,
) -> Result<SaveBatchResult, TvError> {
    // Check if file is in read-only mode due to permission issues
    let permission_state = permission_recovery::get_permission_state(&app_handle, &file_path);
    if permission_state == PermissionState::ReadOnly {
        // Queue all updates for later retry
        for update in &updates {
            permission_recovery::queue_pending_update(
                &app_handle,
                &file_path,
                &table_name,
                update.clone(),
            );
        }

        tracing::warn!(
            component = "tv.commands.save",
            file_path = %file_path,
            update_count = updates.len(),
            "Batch save rejected due to read-only permissions, updates queued"
        );

        return Err(TvError::PermissionDenied {
            path: file_path,
            operation: "write".to_string(),
        });
    }

    state_machine::begin_save(&app_handle, &file_path)?;

    let result = document_writer::save_batch(&TvConfig::default(), &file_path, &table_name, &updates);

    // Handle permission errors by updating state and queueing updates
    if let Err(ref e) = result {
        if matches!(e, TvError::PermissionDenied { .. }) {
            let new_state = permission_recovery::handle_permission_error(&app_handle, &file_path, e);
            if new_state == PermissionState::ReadOnly {
                for update in &updates {
                    permission_recovery::queue_pending_update(
                        &app_handle,
                        &file_path,
                        &table_name,
                        update.clone(),
                    );
                }
            }
        }
    }

    let _ = state_machine::end_save(&app_handle, &file_path, result.is_ok());

    if result.as_ref().is_ok_and(|r| r.success) {
        let column_keys: Vec<&str> = updates.iter().map(|u| u.column_key.as_str()).collect();
        trigger_ability_parse_if_needed(&app_handle, &file_path, &column_keys);
    }

    result
}

/// Tauri command to add a new row to the TOML array-of-tables.
#[tauri::command]
pub fn add_row(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    position: Option<usize>,
    initial_values: Option<HashMap<String, serde_json::Value>>,
) -> Result<AddRowResult, TvError> {
    state_machine::begin_save(&app_handle, &file_path)?;

    let result = document_writer::add_row(&TvConfig::default(), &file_path, &table_name, position, initial_values);
    let _ = state_machine::end_save(&app_handle, &file_path, result.is_ok());

    result
}

fn trigger_ability_parse_if_needed(app_handle: &AppHandle, file_path: &str, column_keys: &[&str]) {
    let has_ability_column = column_keys.iter().any(|key| AbilityParserState::is_ability_column(key));

    if !has_ability_column {
        return;
    }

    if let Some(state) = app_handle.try_state::<Arc<AbilityParserState>>() {
        state.set_tabula_directory(Path::new(file_path));
        state.trigger_parse();
    }
}
