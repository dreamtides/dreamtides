use tauri::{AppHandle, State};

use crate::error::error_types::TvError;
use crate::error::permission_recovery;
use crate::filter::filter_state::FilterStateManager;
use crate::filter::filter_types::{ColumnFilterState, FilterConditionState, FilterState};
use crate::sort::sort_state::SortStateManager;
use crate::sort::sort_types::{SortDirection, SortState};
use crate::sync::state_machine;
use crate::toml::document_loader::{self, TomlTableData};
use crate::toml::metadata;
use crate::toml::metadata_types::FilterCondition;
use crate::traits::TvConfig;

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

    let result = document_loader::load_toml_document(&TvConfig::default(), &file_path, &table_name);

    // Check if this is a "skipped file" scenario (expected, not a real error).
    // Files that are not array-of-tables format are intentionally skipped and
    // should not show as errors in the UI status indicator.
    let is_skipped = matches!(
        &result,
        Err(TvError::TableNotFound { .. }) | Err(TvError::NotAnArrayOfTables { .. })
    );

    // Handle permission errors by updating permission state
    if let Err(ref e) = result {
        permission_recovery::handle_permission_error(&app_handle, &file_path, e);
    } else {
        // On successful load, check and update write permissions
        let perm_state =
            permission_recovery::detect_permission_state(std::path::Path::new(&file_path));
        let message = permission_recovery::get_permission_error_message(perm_state, &file_path);
        permission_recovery::set_permission_state(&app_handle, &file_path, perm_state, &message);
    }

    // Treat skipped files as successful for state machine purposes so we
    // transition to Idle (not Error) and don't show an error indicator.
    state_machine::end_load(&app_handle, &file_path, result.is_ok() || is_skipped);
    result
}

fn restore_sort_state_from_metadata(
    sort_state_manager: &SortStateManager,
    file_path: &str,
    table_name: &str,
) {
    if sort_state_manager.get_sort_state(file_path, table_name).is_some() {
        return;
    }

    match metadata::parse_sort_config_from_file(file_path) {
        Ok(Some(sort_config)) => {
            let direction = if sort_config.ascending {
                SortDirection::Ascending
            } else {
                SortDirection::Descending
            };
            let sort_state = SortState::new(sort_config.column.clone(), direction);
            sort_state_manager.set_sort_state(file_path, table_name, Some(sort_state));
            tracing::debug!(
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

    match metadata::parse_filter_config_from_file(file_path) {
        Ok(Some(filter_config)) => {
            let filter_state = FilterState::new(filter_config.filters.clone(), filter_config.active);
            filter_state_manager.set_filter_state(file_path, table_name, Some(filter_state));

            // Also populate frontend filter states
            if filter_config.active && !filter_config.filters.is_empty() {
                let filter_states: Vec<ColumnFilterState> = filter_config
                    .filters
                    .iter()
                    .map(|f| {
                        let condition = match &f.condition {
                            FilterCondition::Contains(s) => {
                                FilterConditionState::Contains(s.clone())
                            }
                            FilterCondition::Equals(v) => FilterConditionState::Equals(v.clone()),
                            FilterCondition::Range { min, max } => {
                                FilterConditionState::Range { min: *min, max: *max }
                            }
                            FilterCondition::Boolean(b) => FilterConditionState::Boolean(*b),
                            FilterCondition::Values(v) => {
                                FilterConditionState::Values(v.clone())
                            }
                        };
                        ColumnFilterState { column: f.column.clone(), condition }
                    })
                    .collect();
                filter_state_manager.set_filters(file_path, table_name, filter_states);
            }

            tracing::debug!(
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
