use std::collections::HashMap;
use std::sync::RwLock;

use crate::filter::filter_types::{matches_condition, FilterState};
use crate::sort::sort_types::CellValue;
use crate::toml::document_loader::TomlTableData;
use crate::toml::metadata_types::ColumnFilter;

pub struct FilterStateManager {
    states: RwLock<HashMap<String, FilterState>>,
    visibility: RwLock<HashMap<String, Vec<bool>>>,
}

impl FilterStateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            visibility: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_filter_state(&self, file_path: &str, table_name: &str) -> Option<FilterState> {
        let key = format!("{file_path}::{table_name}");
        self.states.read().ok()?.get(&key).cloned()
    }

    pub fn set_filter_state(
        &self,
        file_path: &str,
        table_name: &str,
        state: Option<FilterState>,
    ) -> Option<FilterState> {
        let key = format!("{file_path}::{table_name}");
        let mut states = self.states.write().ok()?;
        match state {
            Some(s) => states.insert(key, s),
            None => states.remove(&key),
        }
    }

    pub fn clear_filter_state(&self, file_path: &str, table_name: &str) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut states) = self.states.write() {
            states.remove(&key);
        }
        if let Ok(mut vis) = self.visibility.write() {
            vis.remove(&key);
        }
    }

    /// Stores a per-row visibility vector for filtered data.
    pub fn set_visibility(&self, file_path: &str, table_name: &str, visible: Vec<bool>) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut vis) = self.visibility.write() {
            vis.insert(key, visible);
        }
    }

    /// Returns the per-row visibility vector, if a filter is active.
    pub fn get_visibility(&self, file_path: &str, table_name: &str) -> Option<Vec<bool>> {
        let key = format!("{file_path}::{table_name}");
        self.visibility.read().ok()?.get(&key).cloned()
    }

    /// Returns true if a specific row is visible under the current filter.
    pub fn is_row_visible(&self, file_path: &str, table_name: &str, row_index: usize) -> bool {
        let key = format!("{file_path}::{table_name}");
        self.visibility
            .read()
            .ok()
            .and_then(|vis| vis.get(&key).and_then(|v| v.get(row_index).copied()))
            .unwrap_or(true)
    }
}

impl Default for FilterStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes row visibility for the given table data and filter set.
///
/// All filters are combined with AND logic: a row is visible only if it
/// passes every filter condition.
pub fn compute_visibility(data: &TomlTableData, filters: &[ColumnFilter]) -> Vec<bool> {
    if filters.is_empty() {
        return vec![true; data.rows.len()];
    }

    let column_indices: Vec<Option<usize>> = filters
        .iter()
        .map(|f| data.headers.iter().position(|h| h == &f.column))
        .collect();

    data.rows
        .iter()
        .map(|row| {
            filters.iter().zip(column_indices.iter()).all(|(filter, col_idx)| {
                let Some(&idx) = col_idx.as_ref() else {
                    return true;
                };
                let cell_value = row
                    .get(idx)
                    .map(CellValue::from_json)
                    .unwrap_or(CellValue::Null);
                matches_condition(&cell_value, &filter.condition)
            })
        })
        .collect()
}

/// Returns the indices of visible rows according to the visibility vector.
pub fn visible_row_indices(visibility: &[bool]) -> Vec<usize> {
    visibility
        .iter()
        .enumerate()
        .filter_map(|(idx, &visible)| if visible { Some(idx) } else { None })
        .collect()
}

/// Filters table data, returning only visible rows.
pub fn apply_filter(data: &TomlTableData, filters: &[ColumnFilter]) -> Vec<usize> {
    let visibility = compute_visibility(data, filters);
    visible_row_indices(&visibility)
}

/// Applies filters to table data and returns the filtered data along with
/// the visibility vector.
pub fn apply_filter_to_data_with_visibility(
    data: TomlTableData,
    filter_state: Option<&FilterState>,
) -> (TomlTableData, Option<Vec<bool>>) {
    match filter_state {
        Some(state) if state.active && !state.filters.is_empty() => {
            let visibility = compute_visibility(&data, &state.filters);
            let filtered_rows: Vec<Vec<serde_json::Value>> = data
                .rows
                .iter()
                .zip(visibility.iter())
                .filter_map(|(row, &visible)| if visible { Some(row.clone()) } else { None })
                .collect();
            (TomlTableData { headers: data.headers, rows: filtered_rows }, Some(visibility))
        }
        _ => (data, None),
    }
}

/// Applies filters to table data and returns the filtered data.
pub fn apply_filter_to_data(
    data: TomlTableData,
    filter_state: Option<&FilterState>,
) -> TomlTableData {
    apply_filter_to_data_with_visibility(data, filter_state).0
}
