use tauri::{AppHandle, State};

use crate::error::error_types::TvError;
use crate::filter::filter_state::{apply_filter_to_data_with_visibility, FilterStateManager};
use crate::filter::filter_types::FilterState;
use crate::sort::sort_state::{apply_sort_to_data_with_mapping, SortStateManager};
use crate::sort::sort_types::{SortDirection, SortState};
use crate::sync::state_machine;
use crate::toml::document_loader::{self, TomlTableData};
use crate::toml::metadata_parser;

/// Tauri command to load a TOML table as spreadsheet data.
#[tauri::command]
pub fn load_toml_table(
    app_handle: AppHandle,
    sort_state_manager: State<SortStateManager>,
    filter_state_manager: State<FilterStateManager>,
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

    restore_sort_state_from_metadata(&sort_state_manager, &file_path, &table_name);
    restore_filter_state_from_metadata(&filter_state_manager, &file_path, &table_name);

    let result = document_loader::load_toml_document(&file_path, &table_name);
    state_machine::end_load(&app_handle, &file_path, result.is_ok());

    result.map(|data| {
        let filter_state = filter_state_manager.get_filter_state(&file_path, &table_name);
        if filter_state.as_ref().is_some_and(|f| f.active) {
            tracing::debug!(
                component = "tv.commands.load",
                file_path = %file_path,
                table_name = %table_name,
                filter_count = ?filter_state.as_ref().map(|f| f.filters.len()),
                "Applying filter to loaded data"
            );
        }
        let (filtered_data, visibility) =
            apply_filter_to_data_with_visibility(data, filter_state.as_ref());
        if let Some(vis) = visibility {
            filter_state_manager.set_visibility(&file_path, &table_name, vis);
        }

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
        let (sorted_data, mapping) =
            apply_sort_to_data_with_mapping(filtered_data, sort_state.as_ref());
        if let Some(indices) = mapping {
            sort_state_manager.set_row_mapping(&file_path, &table_name, indices);
        }
        sorted_data
    })
}

fn restore_sort_state_from_metadata(
    sort_state_manager: &SortStateManager,
    file_path: &str,
    table_name: &str,
) {
    if sort_state_manager.get_sort_state(file_path, table_name).is_some() {
        return;
    }

    match metadata_parser::parse_sort_config_from_file(file_path) {
        Ok(Some(sort_config)) => {
            let direction = if sort_config.ascending {
                SortDirection::Ascending
            } else {
                SortDirection::Descending
            };
            let sort_state = SortState::new(sort_config.column.clone(), direction);
            sort_state_manager.set_sort_state(file_path, table_name, Some(sort_state));
            tracing::info!(
                component = "tv.commands.load",
                file_path = %file_path,
                table_name = %table_name,
                column = %sort_config.column,
                ascending = sort_config.ascending,
                "Restored sort state from metadata"
            );
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(
                component = "tv.commands.load",
                file_path = %file_path,
                error = %e,
                "Failed to parse sort config from metadata, ignoring"
            );
        }
    }
}

fn restore_filter_state_from_metadata(
    filter_state_manager: &FilterStateManager,
    file_path: &str,
    table_name: &str,
) {
    if filter_state_manager.get_filter_state(file_path, table_name).is_some() {
        return;
    }

    match metadata_parser::parse_filter_config_from_file(file_path) {
        Ok(Some(filter_config)) => {
            let filter_state = FilterState::new(filter_config.filters.clone(), filter_config.active);
            filter_state_manager.set_filter_state(file_path, table_name, Some(filter_state));
            tracing::info!(
                component = "tv.commands.load",
                file_path = %file_path,
                table_name = %table_name,
                filter_count = filter_config.filters.len(),
                active = filter_config.active,
                "Restored filter state from metadata"
            );
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(
                component = "tv.commands.load",
                file_path = %file_path,
                error = %e,
                "Failed to parse filter config from metadata, ignoring"
            );
        }
    }
}
