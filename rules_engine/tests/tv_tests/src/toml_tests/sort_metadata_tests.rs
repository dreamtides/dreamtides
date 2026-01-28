use tv_lib::sort::sort_state::{
    apply_sort, apply_sort_to_data, apply_sort_to_data_with_mapping, reorder_rows, SortStateManager,
};
use tv_lib::sort::sort_types::{CellValue, SortDirection, SortState};
use tv_lib::toml::document_loader::TomlTableData;
use tv_lib::toml::metadata::parse_sort_config_from_content;
use tv_lib::toml::metadata_serializer::update_sort_config;
use tv_lib::toml::metadata_types::SortConfig;

use crate::test_utils::mock_filesystem::{MockFileSystem, MockTestConfig};

#[test]
fn test_parse_sort_config_ascending() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.sort]
column = "name"
"#;

    let result = parse_sort_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected sort config to be present");
    assert_eq!(config.column, "name");
    assert!(config.ascending);
}

#[test]
fn test_parse_sort_config_descending() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.sort]
column = "cost"
ascending = false
"#;

    let result = parse_sort_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected sort config to be present");
    assert_eq!(config.column, "cost");
    assert!(!config.ascending);
}

#[test]
fn test_parse_sort_config_missing_metadata() {
    let content = r#"
[[cards]]
id = "card-1"
"#;

    let result = parse_sort_config_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_sort_config_missing_sort_section() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1
"#;

    let result = parse_sort_config_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_sort_config_missing_column() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.sort]
ascending = true
"#;

    let result = parse_sort_config_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_sort_config_adds_sort_section() {
    let mock = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
"#,
    ));
    let sort_config = SortConfig::ascending("name");

    let result = update_sort_config(&mock.config(), "/test.toml", Some(&sort_config));
    assert!(result.is_ok());

    let saved = mock.last_written_content().unwrap();
    assert!(saved.contains("[metadata]"), "Expected [metadata] section in:\n{saved}");
    assert!(saved.contains("column = \"name\""), "Expected column = \"name\" in:\n{saved}");
}

#[test]
fn test_update_sort_config_descending() {
    let mock = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
"#,
    ));
    let sort_config = SortConfig::descending("cost");

    let result = update_sort_config(&mock.config(), "/test.toml", Some(&sort_config));
    assert!(result.is_ok());

    let saved = mock.last_written_content().unwrap();
    assert!(saved.contains("column = \"cost\""), "Expected column = \"cost\" in:\n{saved}");
    assert!(saved.contains("ascending = false"), "Expected ascending = false in:\n{saved}");
}

#[test]
fn test_update_sort_config_removes_sort() {
    let mock = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.sort]
column = "name"
"#,
    ));

    let result = update_sort_config(&mock.config(), "/test.toml", None);
    assert!(result.is_ok());

    let saved = mock.last_written_content().unwrap();
    assert!(!saved.contains("[metadata.sort]"), "Expected no sort section in:\n{saved}");
    assert!(saved.contains("[metadata]"), "Expected metadata section preserved in:\n{saved}");
}

#[test]
fn test_update_sort_config_preserves_other_metadata() {
    let mock = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.columns]]
key = "id"
width = 300
"#,
    ));
    let sort_config = SortConfig::ascending("name");

    let result = update_sort_config(&mock.config(), "/test.toml", Some(&sort_config));
    assert!(result.is_ok());

    let saved = mock.last_written_content().unwrap();
    assert!(saved.contains("key = \"id\""), "Expected column config preserved in:\n{saved}");
    assert!(saved.contains("width = 300"), "Expected width preserved in:\n{saved}");
    assert!(saved.contains("column = \"name\""), "Expected sort config in:\n{saved}");
}

#[test]
fn test_sort_state_manager_per_file() {
    let manager = SortStateManager::new();
    let state_a = SortState::ascending("name".to_string());
    let state_b = SortState::descending("cost".to_string());

    manager.set_sort_state("/file_a.toml", "cards", Some(state_a.clone()));
    manager.set_sort_state("/file_b.toml", "cards", Some(state_b.clone()));

    assert_eq!(manager.get_sort_state("/file_a.toml", "cards"), Some(state_a));
    assert_eq!(manager.get_sort_state("/file_b.toml", "cards"), Some(state_b));
    assert!(manager.get_sort_state("/file_c.toml", "cards").is_none());
}

#[test]
fn test_apply_sort_preserves_original_data() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(3)],
            vec![serde_json::json!("Alice"), serde_json::json!(1)],
            vec![serde_json::json!("Bob"), serde_json::json!(2)],
        ],
    };

    let sort_state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![1, 2, 0]);

    assert_eq!(data.rows[0][0], serde_json::json!("Charlie"));
    assert_eq!(data.rows[1][0], serde_json::json!("Alice"));
    assert_eq!(data.rows[2][0], serde_json::json!("Bob"));
}

#[test]
fn test_apply_sort_to_data_none_returns_unchanged() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Charlie")], vec![serde_json::json!("Alice")]],
    };

    let result = apply_sort_to_data(data, None);
    assert_eq!(result.rows[0][0], serde_json::json!("Charlie"));
    assert_eq!(result.rows[1][0], serde_json::json!("Alice"));
}

#[test]
fn test_sort_with_null_values() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Bob")], vec![serde_json::json!(null)], vec![
            serde_json::json!("Alice"),
        ]],
    };

    let sort_state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    let reordered = reorder_rows(&data, &indices);

    assert_eq!(reordered[0][0], serde_json::json!("Alice"));
    assert_eq!(reordered[1][0], serde_json::json!("Bob"));
    assert_eq!(reordered[2][0], serde_json::json!(null));
}

#[test]
fn test_sort_mixed_types() {
    let data = TomlTableData {
        headers: vec!["value".to_string()],
        rows: vec![
            vec![serde_json::json!("text")],
            vec![serde_json::json!(42)],
            vec![serde_json::json!(true)],
            vec![serde_json::json!(null)],
        ],
    };

    let sort_state = SortState::ascending("value".to_string());
    let indices = apply_sort(&data, &sort_state);
    let reordered = reorder_rows(&data, &indices);

    assert_eq!(reordered[0][0], serde_json::json!(true));
    assert_eq!(reordered[1][0], serde_json::json!(42));
    assert_eq!(reordered[2][0], serde_json::json!("text"));
    assert_eq!(reordered[3][0], serde_json::json!(null));
}

#[test]
fn test_cell_value_numeric_comparison() {
    let int_val = CellValue::Integer(5);
    let float_val = CellValue::Float(3.7);
    assert!(int_val > float_val);

    let same_int = CellValue::Integer(3);
    let same_float = CellValue::Float(3.0);
    assert_eq!(same_int.cmp_values(&same_float), std::cmp::Ordering::Equal);
}

#[test]
fn test_cell_value_case_insensitive_string_sort() {
    let upper = CellValue::String("Banana".to_string());
    let lower = CellValue::String("apple".to_string());
    assert!(lower < upper);
}

#[test]
fn test_sort_direction_toggle() {
    assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
    assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
}

#[test]
fn test_row_mapping_stored_and_retrieved() {
    let manager = SortStateManager::new();
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());

    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    assert_eq!(manager.get_row_mapping("/test.toml", "cards"), Some(vec![2, 0, 1]));
}

#[test]
fn test_display_to_original_translates_correctly() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);

    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 2);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 1), 0);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 2), 1);
}

#[test]
fn test_display_to_original_passthrough_without_mapping() {
    let manager = SortStateManager::new();

    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 0);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 5), 5);
}

#[test]
fn test_display_to_original_passthrough_out_of_bounds() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);

    assert_eq!(manager.display_to_original("/test.toml", "cards", 10), 10);
}

#[test]
fn test_clear_sort_state_also_clears_row_mapping() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/test.toml", "cards", Some(SortState::ascending("name".to_string())));
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);

    manager.clear_sort_state("/test.toml", "cards");
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());
}

#[test]
fn test_apply_sort_to_data_with_mapping_returns_indices() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(3)],
            vec![serde_json::json!("Alice"), serde_json::json!(1)],
            vec![serde_json::json!("Bob"), serde_json::json!(2)],
        ],
    };

    let sort_state = SortState::ascending("name".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&sort_state));

    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));

    let mapping = mapping.expect("Expected mapping for sorted data");
    assert_eq!(mapping, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_to_data_with_mapping_none_returns_no_mapping() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Charlie")], vec![serde_json::json!("Alice")]],
    };

    let (result, mapping) = apply_sort_to_data_with_mapping(data, None);
    assert_eq!(result.rows[0][0], serde_json::json!("Charlie"));
    assert!(mapping.is_none());
}

#[test]
fn test_row_mapping_per_file_isolation() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/file_a.toml", "cards", vec![2, 0, 1]);
    manager.set_row_mapping("/file_b.toml", "cards", vec![1, 0]);

    assert_eq!(manager.get_row_mapping("/file_a.toml", "cards"), Some(vec![2, 0, 1]));
    assert_eq!(manager.get_row_mapping("/file_b.toml", "cards"), Some(vec![1, 0]));
    assert!(manager.get_row_mapping("/file_c.toml", "cards").is_none());
}

#[test]
fn test_sort_by_string_descending() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "age".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(30), serde_json::json!(true)],
            vec![serde_json::json!("Alice"), serde_json::json!(25), serde_json::json!(false)],
            vec![serde_json::json!("Bob"), serde_json::json!(35), serde_json::json!(true)],
        ],
    };

    let sort_state = SortState::descending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 2, 1]);
}

#[test]
fn test_sort_by_number_column() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "age".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(30)],
            vec![serde_json::json!("Alice"), serde_json::json!(25)],
            vec![serde_json::json!("Bob"), serde_json::json!(35)],
        ],
    };

    let sort_state = SortState::ascending("age".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![1, 0, 2]);
}

#[test]
fn test_sort_by_boolean_column() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(true)],
            vec![serde_json::json!("Alice"), serde_json::json!(false)],
            vec![serde_json::json!("Bob"), serde_json::json!(true)],
        ],
    };

    let sort_state = SortState::ascending("active".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![1, 0, 2]);
}

#[test]
fn test_sort_nonexistent_column_returns_identity() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Charlie")], vec![serde_json::json!("Alice")], vec![
            serde_json::json!("Bob"),
        ]],
    };

    let sort_state = SortState::ascending("nonexistent".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_reorder_rows_by_indices() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Charlie")], vec![serde_json::json!("Alice")], vec![
            serde_json::json!("Bob"),
        ]],
    };

    let indices = vec![2, 0, 1];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!("Bob"));
    assert_eq!(reordered[1][0], serde_json::json!("Charlie"));
    assert_eq!(reordered[2][0], serde_json::json!("Alice"));
}

#[test]
fn test_apply_sort_to_data_sorts_rows() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Charlie")], vec![serde_json::json!("Alice")], vec![
            serde_json::json!("Bob"),
        ]],
    };

    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));
}

#[test]
fn test_cell_value_nulls_sort_last() {
    let null = CellValue::Null;
    let number = CellValue::Integer(5);
    assert!(null > number);
}

#[test]
fn test_cell_value_integer_vs_float_ordering() {
    let i = CellValue::Integer(5);
    let f = CellValue::Float(5.5);
    assert!(i < f);
}

#[test]
fn test_cell_value_boolean_ordering() {
    let t = CellValue::Boolean(true);
    let f = CellValue::Boolean(false);
    assert!(f < t);
}

#[test]
fn test_row_mapping_end_to_end_sort_and_translate() {
    let manager = SortStateManager::new();

    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(3)],
            vec![serde_json::json!("Alice"), serde_json::json!(1)],
            vec![serde_json::json!("Bob"), serde_json::json!(2)],
        ],
    };

    let sort_state = SortState::ascending("name".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&sort_state));
    manager.set_row_mapping("/test.toml", "cards", mapping.unwrap());

    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 1);

    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 1), 2);

    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 2), 0);
}
