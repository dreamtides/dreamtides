use std::collections::HashMap;
use std::sync::RwLock;

use crate::filter::filter_types::{matches_condition, ColumnFilterState, FilterConditionState, FilterState};
use crate::sort::sort_types::CellValue;
use crate::toml::document_loader::TomlTableData;
use crate::toml::metadata_types::ColumnFilter;

pub struct FilterStateManager {
    states: RwLock<HashMap<String, FilterState>>,
    visibility: RwLock<HashMap<String, Vec<bool>>>,
    filter_states: RwLock<HashMap<String, Vec<ColumnFilterState>>>,
    hidden_rows: RwLock<HashMap<String, Vec<usize>>>,
}

impl FilterStateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            visibility: RwLock::new(HashMap::new()),
            filter_states: RwLock::new(HashMap::new()),
            hidden_rows: RwLock::new(HashMap::new()),
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

    /// Returns the active filters for a given file and table.
    pub fn get_filters(&self, file_path: &str, table_name: &str) -> Vec<ColumnFilterState> {
        let key = format!("{file_path}::{table_name}");
        self.filter_states.read().ok().and_then(|s| s.get(&key).cloned()).unwrap_or_default()
    }

    /// Sets the active filters for a given file and table.
    pub fn set_filters(&self, file_path: &str, table_name: &str, filters: Vec<ColumnFilterState>) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut states) = self.filter_states.write() {
            if filters.is_empty() {
                states.remove(&key);
            } else {
                states.insert(key, filters);
            }
        }
    }

    /// Clears all filters for a given file and table.
    pub fn clear_filters(&self, file_path: &str, table_name: &str) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut states) = self.filter_states.write() {
            states.remove(&key);
        }
        if let Ok(mut hidden) = self.hidden_rows.write() {
            hidden.remove(&key);
        }
    }

    /// Stores the set of hidden (filtered-out) row indices.
    pub fn set_hidden_rows(&self, file_path: &str, table_name: &str, rows: Vec<usize>) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut hidden) = self.hidden_rows.write() {
            if rows.is_empty() {
                hidden.remove(&key);
            } else {
                hidden.insert(key, rows);
            }
        }
    }

    /// Returns the set of hidden row indices, if any filter is active.
    pub fn get_hidden_rows(&self, file_path: &str, table_name: &str) -> Vec<usize> {
        let key = format!("{file_path}::{table_name}");
        self.hidden_rows.read().ok().and_then(|h| h.get(&key).cloned()).unwrap_or_default()
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

/// Applies filters to table data and returns the indices of rows that should be hidden.
pub fn compute_hidden_rows(data: &TomlTableData, filters: &[ColumnFilterState]) -> Vec<usize> {
    if filters.is_empty() {
        return Vec::new();
    }

    let mut hidden = Vec::new();

    for (row_idx, row) in data.rows.iter().enumerate() {
        let mut visible = true;
        for filter in filters {
            let col_idx = match data.headers.iter().position(|h| h == &filter.column) {
                Some(idx) => idx,
                None => continue,
            };

            let cell_value = row.get(col_idx).unwrap_or(&serde_json::Value::Null);
            if !matches_filter(cell_value, &filter.condition) {
                visible = false;
                break;
            }
        }
        if !visible {
            hidden.push(row_idx);
        }
    }

    hidden
}

/// Returns the visible row indices after applying filters.
pub fn compute_visible_rows(data: &TomlTableData, filters: &[ColumnFilterState]) -> Vec<usize> {
    if filters.is_empty() {
        return (0..data.rows.len()).collect();
    }

    let hidden = compute_hidden_rows(data, filters);
    (0..data.rows.len()).filter(|idx| !hidden.contains(idx)).collect()
}

fn matches_filter(value: &serde_json::Value, condition: &FilterConditionState) -> bool {
    match condition {
        FilterConditionState::Contains(substring) => {
            let text = value_to_string(value);
            text.to_lowercase().contains(&substring.to_lowercase())
        }
        FilterConditionState::Equals(expected) => value == expected,
        FilterConditionState::Range { min, max } => {
            let num = value_to_f64(value);
            match num {
                Some(n) => {
                    min.is_none_or(|m| n >= m) && max.is_none_or(|m| n <= m)
                }
                None => false,
            }
        }
        FilterConditionState::Boolean(expected) => match value {
            serde_json::Value::Bool(b) => b == expected,
            _ => false,
        },
        FilterConditionState::Values(allowed) => {
            if allowed.is_empty() {
                return true;
            }
            allowed.iter().any(|a| values_match(value, a))
        }
    }
}

fn values_match(value: &serde_json::Value, allowed: &serde_json::Value) -> bool {
    if value == allowed {
        return true;
    }
    value_to_string(value) == value_to_string(allowed)
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => format!("{arr:?}"),
        serde_json::Value::Object(obj) => format!("{obj:?}"),
    }
}

fn value_to_f64(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}
