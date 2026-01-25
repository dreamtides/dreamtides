use tauri::{AppHandle, State};

use crate::error::error_types::TvError;
use crate::sort::sort_state::{apply_sort_to_data, SortStateManager};
use crate::sync::state_machine;
use crate::toml::document_loader::{self, TomlTableData};

/// Tauri command to load a TOML table as spreadsheet data.
#[tauri::command]
pub fn load_toml_table(
    app_handle: AppHandle,
    sort_state_manager: State<SortStateManager>,
    file_path: String,
    table_name: String,
) -> Result<TomlTableData, TvError> {
    if let Err(e) = state_machine::begin_load(&app_handle, &file_path) {
        tracing::warn!(
            component = "tv.commands.load",
            file_path = %file_path,
            error = %e,
            "Could not transition to Loading state, proceeding anyway"
        );
    }

    let result = document_loader::load_toml_document(&file_path, &table_name);
    state_machine::end_load(&app_handle, &file_path, result.is_ok());

    result.map(|data| {
        let sort_state = sort_state_manager.get_sort_state(&file_path, &table_name);
        if sort_state.is_some() {
            tracing::debug!(
                component = "tv.commands.load",
                file_path = %file_path,
                table_name = %table_name,
                sort_column = ?sort_state.as_ref().map(|s| &s.column),
                "Applying sort to loaded data"
            );
        }
        apply_sort_to_data(data, sort_state.as_ref())
    })
}
