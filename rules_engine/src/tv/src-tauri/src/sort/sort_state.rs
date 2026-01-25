use std::collections::HashMap;
use std::sync::RwLock;

use crate::sort::sort_types::{CellValue, SortDirection, SortState};
use crate::toml::document_loader::TomlTableData;

pub struct SortStateManager {
    states: RwLock<HashMap<String, SortState>>,
}

impl SortStateManager {
    pub fn new() -> Self {
        Self { states: RwLock::new(HashMap::new()) }
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

pub fn apply_sort_to_data(data: TomlTableData, sort_state: Option<&SortState>) -> TomlTableData {
    match sort_state {
        Some(state) => {
            let indices = apply_sort(&data, state);
            let sorted_rows = reorder_rows(&data, &indices);
            TomlTableData { headers: data.headers, rows: sorted_rows }
        }
        None => data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_test_data() -> TomlTableData {
        TomlTableData {
            headers: vec!["name".to_string(), "age".to_string(), "active".to_string()],
            rows: vec![
                vec![json!("Charlie"), json!(30), json!(true)],
                vec![json!("Alice"), json!(25), json!(false)],
                vec![json!("Bob"), json!(35), json!(true)],
            ],
        }
    }

    #[test]
    fn test_sort_by_string_ascending() {
        let data = make_test_data();
        let sort_state = SortState::ascending("name".to_string());
        let indices = apply_sort(&data, &sort_state);
        assert_eq!(indices, vec![1, 2, 0]);
    }

    #[test]
    fn test_sort_by_string_descending() {
        let data = make_test_data();
        let sort_state = SortState::descending("name".to_string());
        let indices = apply_sort(&data, &sort_state);
        assert_eq!(indices, vec![0, 2, 1]);
    }

    #[test]
    fn test_sort_by_number() {
        let data = make_test_data();
        let sort_state = SortState::ascending("age".to_string());
        let indices = apply_sort(&data, &sort_state);
        assert_eq!(indices, vec![1, 0, 2]);
    }

    #[test]
    fn test_sort_by_boolean() {
        let data = make_test_data();
        let sort_state = SortState::ascending("active".to_string());
        let indices = apply_sort(&data, &sort_state);
        assert_eq!(indices, vec![1, 0, 2]);
    }

    #[test]
    fn test_sort_nonexistent_column() {
        let data = make_test_data();
        let sort_state = SortState::ascending("nonexistent".to_string());
        let indices = apply_sort(&data, &sort_state);
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_reorder_rows() {
        let data = make_test_data();
        let indices = vec![2, 0, 1];
        let reordered = reorder_rows(&data, &indices);
        assert_eq!(reordered[0][0], json!("Bob"));
        assert_eq!(reordered[1][0], json!("Charlie"));
        assert_eq!(reordered[2][0], json!("Alice"));
    }

    #[test]
    fn test_apply_sort_to_data() {
        let data = make_test_data();
        let sort_state = SortState::ascending("name".to_string());
        let sorted = apply_sort_to_data(data, Some(&sort_state));
        assert_eq!(sorted.rows[0][0], json!("Alice"));
        assert_eq!(sorted.rows[1][0], json!("Bob"));
        assert_eq!(sorted.rows[2][0], json!("Charlie"));
    }

    #[test]
    fn test_state_manager() {
        let manager = SortStateManager::new();
        let state = SortState::ascending("name".to_string());

        assert!(manager.get_sort_state("/path/file.toml", "cards").is_none());

        manager.set_sort_state("/path/file.toml", "cards", Some(state.clone()));
        assert_eq!(manager.get_sort_state("/path/file.toml", "cards"), Some(state));

        manager.clear_sort_state("/path/file.toml", "cards");
        assert!(manager.get_sort_state("/path/file.toml", "cards").is_none());
    }
}
