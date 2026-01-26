use tauri::State;

use crate::filter::filter_state::FilterStateManager;
use crate::filter::filter_types::FilterState;
use crate::toml::metadata_types::{ColumnFilter, FilterCondition, FilterConfig};

#[derive(serde::Deserialize)]
pub struct SetFilterRequest {
    pub filters: Vec<ColumnFilterRequest>,
    pub active: bool,
}

#[derive(serde::Deserialize)]
pub struct ColumnFilterRequest {
    pub column: String,
    pub condition: FilterCondition,
}

#[derive(serde::Serialize)]
pub struct FilterStateResponse {
    pub filters: Vec<FilterEntry>,
    pub active: bool,
}

#[derive(serde::Serialize)]
pub struct FilterEntry {
    pub column: String,
    pub condition: FilterCondition,
}

impl From<Option<FilterState>> for FilterStateResponse {
    fn from(state: Option<FilterState>) -> Self {
        match state {
            Some(s) => FilterStateResponse {
                filters: s
                    .filters
                    .into_iter()
                    .map(|f| FilterEntry { column: f.column, condition: f.condition })
                    .collect(),
                active: s.active,
            },
            None => FilterStateResponse { filters: Vec::new(), active: false },
        }
    }
}

#[tauri::command]
pub fn get_filter_state(
    state: State<FilterStateManager>,
    file_path: String,
    table_name: String,
) -> FilterStateResponse {
    let filter_state = state.get_filter_state(&file_path, &table_name);
    tracing::debug!(
        component = "tv.commands.filter",
        file_path = %file_path,
        table_name = %table_name,
        has_filter = filter_state.is_some(),
        "Get filter state"
    );
    FilterStateResponse::from(filter_state)
}

#[tauri::command]
pub fn set_filter_state(
    state: State<FilterStateManager>,
    file_path: String,
    table_name: String,
    filter: Option<SetFilterRequest>,
) -> FilterStateResponse {
    let new_state = filter.map(|f| {
        let filters = f
            .filters
            .into_iter()
            .map(|cf| ColumnFilter::new(cf.column, cf.condition))
            .collect();
        FilterState::new(filters, f.active)
    });

    tracing::info!(
        component = "tv.commands.filter",
        file_path = %file_path,
        table_name = %table_name,
        filter_count = ?new_state.as_ref().map(|s| s.filters.len()),
        active = ?new_state.as_ref().map(|s| s.active),
        "Set filter state"
    );

    state.set_filter_state(&file_path, &table_name, new_state.clone());
    persist_filter_to_metadata(&file_path, new_state.as_ref());
    FilterStateResponse::from(new_state)
}

#[tauri::command]
pub fn clear_filter_state(
    state: State<FilterStateManager>,
    file_path: String,
    table_name: String,
) -> FilterStateResponse {
    tracing::info!(
        component = "tv.commands.filter",
        file_path = %file_path,
        table_name = %table_name,
        "Clear filter state"
    );
    state.clear_filter_state(&file_path, &table_name);
    persist_filter_to_metadata(&file_path, None);
    FilterStateResponse { filters: Vec::new(), active: false }
}

#[tauri::command]
pub fn get_filter_visibility(
    state: State<FilterStateManager>,
    file_path: String,
    table_name: String,
) -> Vec<bool> {
    let visibility = state.get_visibility(&file_path, &table_name);
    tracing::debug!(
        component = "tv.commands.filter",
        file_path = %file_path,
        table_name = %table_name,
        has_visibility = visibility.is_some(),
        "Get filter visibility"
    );
    visibility.unwrap_or_default()
}

#[tauri::command]
pub fn is_row_visible(
    state: State<FilterStateManager>,
    file_path: String,
    table_name: String,
    row_index: usize,
) -> bool {
    state.is_row_visible(&file_path, &table_name, row_index)
}

fn persist_filter_to_metadata(file_path: &str, filter_state: Option<&FilterState>) {
    let filter_config = filter_state.map(|s| FilterConfig {
        filters: s.filters.clone(),
        active: s.active,
    });

    if let Err(e) =
        crate::toml::metadata_serializer::update_filter_config(file_path, filter_config.as_ref())
    {
        tracing::warn!(
            component = "tv.commands.filter",
            file_path = %file_path,
            error = %e,
            "Failed to persist filter state to metadata"
        );
    }
}
