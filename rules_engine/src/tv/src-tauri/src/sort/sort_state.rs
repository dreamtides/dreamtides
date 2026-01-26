use std::collections::HashMap;
use std::sync::RwLock;

use crate::sort::sort_types::{CellValue, SortDirection, SortState};
use crate::toml::document_loader::TomlTableData;

pub struct SortStateManager {
    states: RwLock<HashMap<String, SortState>>,
    row_mappings: RwLock<HashMap<String, Vec<usize>>>,
}

impl SortStateManager {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            row_mappings: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_sort_state(&self, file_path: &str, table_name: &str) -> Option<SortState> {
        let key = format!("{file_path}::{table_name}");
        self.states.read().ok()?.get(&key).cloned()
    }

    pub fn set_sort_state(
        &self,
        file_path: &str,
        table_name: &str,
        state: Option<SortState>,
    ) -> Option<SortState> {
        let key = format!("{file_path}::{table_name}");
        let mut states = self.states.write().ok()?;
        match state {
            Some(s) => states.insert(key, s),
            None => states.remove(&key),
        }
    }

    pub fn clear_sort_state(&self, file_path: &str, table_name: &str) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut states) = self.states.write() {
            states.remove(&key);
        }
        if let Ok(mut mappings) = self.row_mappings.write() {
            mappings.remove(&key);
        }
    }

    /// Stores a display-to-original row index mapping for a sorted table.
    pub fn set_row_mapping(&self, file_path: &str, table_name: &str, mapping: Vec<usize>) {
        let key = format!("{file_path}::{table_name}");
        if let Ok(mut mappings) = self.row_mappings.write() {
            mappings.insert(key, mapping);
        }
    }

    /// Returns the display-to-original row index mapping, if a sort is active.
    pub fn get_row_mapping(&self, file_path: &str, table_name: &str) -> Option<Vec<usize>> {
        let key = format!("{file_path}::{table_name}");
        self.row_mappings.read().ok()?.get(&key).cloned()
    }

    /// Translates a display row index to the original TOML row index.
    pub fn display_to_original(&self, file_path: &str, table_name: &str, display_index: usize) -> usize {
        let key = format!("{file_path}::{table_name}");
        self.row_mappings
            .read()
            .ok()
            .and_then(|mappings| mappings.get(&key).and_then(|m| m.get(display_index).copied()))
            .unwrap_or(display_index)
    }
}

impl Default for SortStateManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_sort(data: &TomlTableData, sort_state: &SortState) -> Vec<usize> {
    let column_index = match data.headers.iter().position(|h| h == &sort_state.column) {
        Some(idx) => idx,
        None => return (0..data.rows.len()).collect(),
    };

    let mut indices: Vec<usize> = (0..data.rows.len()).collect();

    indices.sort_by(|&a, &b| {
        let val_a = data.rows.get(a).and_then(|row| row.get(column_index));
        let val_b = data.rows.get(b).and_then(|row| row.get(column_index));

        let cell_a = val_a.map(CellValue::from_json).unwrap_or(CellValue::Null);
        let cell_b = val_b.map(CellValue::from_json).unwrap_or(CellValue::Null);

        let ordering = cell_a.cmp_values(&cell_b);

        match sort_state.direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });

    indices
}

pub fn reorder_rows(data: &TomlTableData, indices: &[usize]) -> Vec<Vec<serde_json::Value>> {
    indices.iter().filter_map(|&idx| data.rows.get(idx).cloned()).collect()
}

/// Sorts table data and returns both the sorted data and the display-to-original index mapping.
pub fn apply_sort_to_data_with_mapping(
    data: TomlTableData,
    sort_state: Option<&SortState>,
) -> (TomlTableData, Option<Vec<usize>>) {
    match sort_state {
        Some(state) => {
            let indices = apply_sort(&data, state);
            let sorted_rows = reorder_rows(&data, &indices);
            (TomlTableData { headers: data.headers, rows: sorted_rows }, Some(indices))
        }
        None => (data, None),
    }
}

pub fn apply_sort_to_data(data: TomlTableData, sort_state: Option<&SortState>) -> TomlTableData {
    apply_sort_to_data_with_mapping(data, sort_state).0
}
