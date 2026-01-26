use tauri::State;

use crate::sort::sort_state::SortStateManager;
use crate::sort::sort_types::{SortDirection, SortState};
use crate::toml::metadata_types::SortConfig;

#[derive(serde::Deserialize)]
pub struct SetSortRequest {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(serde::Serialize)]
pub struct SortStateResponse {
    pub column: Option<String>,
    pub direction: Option<SortDirection>,
}

impl From<Option<SortState>> for SortStateResponse {
    fn from(state: Option<SortState>) -> Self {
        match state {
            Some(s) => SortStateResponse { column: Some(s.column), direction: Some(s.direction) },
            None => SortStateResponse { column: None, direction: None },
        }
    }
}

#[tauri::command]
pub fn get_sort_state(
    state: State<SortStateManager>,
    file_path: String,
    table_name: String,
) -> SortStateResponse {
    let sort_state = state.get_sort_state(&file_path, &table_name);
    tracing::debug!(
        component = "tv.commands.sort",
        file_path = %file_path,
        table_name = %table_name,
        has_sort = sort_state.is_some(),
        "Get sort state"
    );
    SortStateResponse::from(sort_state)
}

#[tauri::command]
pub fn set_sort_state(
    state: State<SortStateManager>,
    file_path: String,
    table_name: String,
    sort: Option<SetSortRequest>,
) -> SortStateResponse {
    let new_state = sort.map(|s| SortState::new(s.column, s.direction));

    tracing::info!(
        component = "tv.commands.sort",
        file_path = %file_path,
        table_name = %table_name,
        column = ?new_state.as_ref().map(|s| &s.column),
        direction = ?new_state.as_ref().map(|s| s.direction),
        "Set sort state"
    );

    state.set_sort_state(&file_path, &table_name, new_state.clone());
    persist_sort_to_metadata(&file_path, new_state.as_ref());
    SortStateResponse::from(new_state)
}

#[tauri::command]
pub fn clear_sort_state(
    state: State<SortStateManager>,
    file_path: String,
    table_name: String,
) -> SortStateResponse {
    tracing::info!(
        component = "tv.commands.sort",
        file_path = %file_path,
        table_name = %table_name,
        "Clear sort state"
    );
    state.clear_sort_state(&file_path, &table_name);
    persist_sort_to_metadata(&file_path, None);
    SortStateResponse { column: None, direction: None }
}

#[tauri::command]
pub fn get_sort_row_mapping(
    state: State<SortStateManager>,
    file_path: String,
    table_name: String,
) -> Vec<usize> {
    let mapping = state.get_row_mapping(&file_path, &table_name);
    tracing::debug!(
        component = "tv.commands.sort",
        file_path = %file_path,
        table_name = %table_name,
        has_mapping = mapping.is_some(),
        "Get sort row mapping"
    );
    mapping.unwrap_or_default()
}

#[tauri::command]
pub fn translate_row_index(
    state: State<SortStateManager>,
    file_path: String,
    table_name: String,
    display_index: usize,
) -> usize {
    state.display_to_original(&file_path, &table_name, display_index)
}

fn persist_sort_to_metadata(file_path: &str, sort_state: Option<&SortState>) {
    let sort_config = sort_state.map(|s| {
        let ascending = s.direction == SortDirection::Ascending;
        SortConfig { column: s.column.clone(), ascending }
    });

    if let Err(e) = crate::toml::metadata_serializer::update_sort_config(file_path, sort_config.as_ref()) {
        tracing::warn!(
            component = "tv.commands.sort",
            file_path = %file_path,
            error = %e,
            "Failed to persist sort state to metadata"
        );
    }
}
