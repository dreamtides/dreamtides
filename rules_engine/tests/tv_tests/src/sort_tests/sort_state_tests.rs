use tv_lib::sort::sort_state::{
    apply_sort, apply_sort_to_data, apply_sort_to_data_with_mapping, reorder_rows, SortStateManager,
};
use tv_lib::sort::sort_types::SortState;
use tv_lib::toml::document_loader::TomlTableData;

fn sample_three_row_data() -> TomlTableData {
    TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(3)],
            vec![serde_json::json!("Alice"), serde_json::json!(1)],
            vec![serde_json::json!("Bob"), serde_json::json!(2)],
        ],
    }
}

#[test]
fn test_sort_state_manager_new_returns_empty() {
    let manager = SortStateManager::new();
    assert!(manager.get_sort_state("/any.toml", "cards").is_none());
}

#[test]
fn test_sort_state_manager_default_returns_empty() {
    let manager = SortStateManager::default();
    assert!(manager.get_sort_state("/any.toml", "cards").is_none());
}

#[test]
fn test_sort_state_manager_set_and_get() {
    let manager = SortStateManager::new();
    let state = SortState::ascending("name".to_string());
    manager.set_sort_state("/test.toml", "cards", Some(state.clone()));
    assert_eq!(manager.get_sort_state("/test.toml", "cards"), Some(state));
}

#[test]
fn test_sort_state_manager_set_none_removes() {
    let manager = SortStateManager::new();
    let state = SortState::ascending("name".to_string());
    manager.set_sort_state("/test.toml", "cards", Some(state));
    manager.set_sort_state("/test.toml", "cards", None);
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
}

#[test]
fn test_sort_state_manager_per_file_isolation() {
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
fn test_sort_state_manager_per_table_isolation() {
    let manager = SortStateManager::new();
    let state_a = SortState::ascending("name".to_string());
    let state_b = SortState::descending("cost".to_string());

    manager.set_sort_state("/test.toml", "cards", Some(state_a.clone()));
    manager.set_sort_state("/test.toml", "abilities", Some(state_b.clone()));

    assert_eq!(manager.get_sort_state("/test.toml", "cards"), Some(state_a));
    assert_eq!(manager.get_sort_state("/test.toml", "abilities"), Some(state_b));
}

#[test]
fn test_sort_state_manager_overwrite() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/test.toml", "cards", Some(SortState::ascending("name".to_string())));
    let new_state = SortState::descending("cost".to_string());
    manager.set_sort_state("/test.toml", "cards", Some(new_state.clone()));
    assert_eq!(manager.get_sort_state("/test.toml", "cards"), Some(new_state));
}

#[test]
fn test_sort_state_manager_set_returns_previous_value() {
    let manager = SortStateManager::new();
    let result = manager.set_sort_state(
        "/test.toml",
        "cards",
        Some(SortState::ascending("name".to_string())),
    );
    assert!(result.is_none());

    let result = manager.set_sort_state(
        "/test.toml",
        "cards",
        Some(SortState::descending("cost".to_string())),
    );
    assert_eq!(result, Some(SortState::ascending("name".to_string())));
}

#[test]
fn test_clear_sort_state() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/test.toml", "cards", Some(SortState::ascending("name".to_string())));
    manager.clear_sort_state("/test.toml", "cards");
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
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
fn test_clear_sort_state_no_effect_on_nonexistent() {
    let manager = SortStateManager::new();
    manager.clear_sort_state("/nonexistent.toml", "cards");
    assert!(manager.get_sort_state("/nonexistent.toml", "cards").is_none());
}

#[test]
fn test_clear_sort_state_only_clears_target() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/a.toml", "cards", Some(SortState::ascending("name".to_string())));
    manager.set_sort_state("/b.toml", "cards", Some(SortState::descending("cost".to_string())));

    manager.clear_sort_state("/a.toml", "cards");
    assert!(manager.get_sort_state("/a.toml", "cards").is_none());
    assert!(manager.get_sort_state("/b.toml", "cards").is_some());
}

#[test]
fn test_row_mapping_set_and_get() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    assert_eq!(manager.get_row_mapping("/test.toml", "cards"), Some(vec![2, 0, 1]));
}

#[test]
fn test_row_mapping_returns_none_when_unset() {
    let manager = SortStateManager::new();
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());
}

#[test]
fn test_row_mapping_per_file_isolation() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/a.toml", "cards", vec![2, 0, 1]);
    manager.set_row_mapping("/b.toml", "cards", vec![1, 0]);

    assert_eq!(manager.get_row_mapping("/a.toml", "cards"), Some(vec![2, 0, 1]));
    assert_eq!(manager.get_row_mapping("/b.toml", "cards"), Some(vec![1, 0]));
    assert!(manager.get_row_mapping("/c.toml", "cards").is_none());
}

#[test]
fn test_row_mapping_overwrite() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    manager.set_row_mapping("/test.toml", "cards", vec![0, 1, 2]);
    assert_eq!(manager.get_row_mapping("/test.toml", "cards"), Some(vec![0, 1, 2]));
}

#[test]
fn test_row_mapping_empty_vector() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![]);
    assert_eq!(manager.get_row_mapping("/test.toml", "cards"), Some(vec![]));
}

#[test]
fn test_display_to_original_translates() {
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
    assert_eq!(manager.display_to_original("/test.toml", "cards", 100), 100);
}

#[test]
fn test_display_to_original_out_of_bounds_returns_passthrough() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 10), 10);
}

#[test]
fn test_display_to_original_wrong_file_returns_passthrough() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    assert_eq!(manager.display_to_original("/other.toml", "cards", 0), 0);
}

#[test]
fn test_apply_sort_ascending_by_string() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_descending_by_string() {
    let data = sample_three_row_data();
    let sort_state = SortState::descending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 2, 1]);
}

#[test]
fn test_apply_sort_ascending_by_number() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("cost".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_descending_by_number() {
    let data = sample_three_row_data();
    let sort_state = SortState::descending("cost".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 2, 1]);
}

#[test]
fn test_apply_sort_nonexistent_column_returns_identity() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("nonexistent".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_apply_sort_empty_rows() {
    let data = TomlTableData { headers: vec!["name".to_string()], rows: vec![] };
    let sort_state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert!(indices.is_empty());
}

#[test]
fn test_apply_sort_single_row() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Only")]],
    };
    let sort_state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0]);
}

#[test]
fn test_apply_sort_with_null_values() {
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
fn test_apply_sort_with_mixed_types() {
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
fn test_apply_sort_by_boolean_column() {
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
fn test_apply_sort_preserves_original_data() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let _indices = apply_sort(&data, &sort_state);
    assert_eq!(data.rows[0][0], serde_json::json!("Charlie"));
    assert_eq!(data.rows[1][0], serde_json::json!("Alice"));
    assert_eq!(data.rows[2][0], serde_json::json!("Bob"));
}

#[test]
fn test_apply_sort_stable_for_equal_values() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "score".to_string()],
        rows: vec![
            vec![serde_json::json!("First"), serde_json::json!(10)],
            vec![serde_json::json!("Second"), serde_json::json!(10)],
            vec![serde_json::json!("Third"), serde_json::json!(10)],
        ],
    };
    let sort_state = SortState::ascending("score".to_string());
    let indices = apply_sort(&data, &sort_state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_reorder_rows_reorders_correctly() {
    let data = sample_three_row_data();
    let indices = vec![2, 0, 1];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!("Bob"));
    assert_eq!(reordered[1][0], serde_json::json!("Charlie"));
    assert_eq!(reordered[2][0], serde_json::json!("Alice"));
}

#[test]
fn test_reorder_rows_identity_indices() {
    let data = sample_three_row_data();
    let indices = vec![0, 1, 2];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered, data.rows);
}

#[test]
fn test_reorder_rows_reverse_indices() {
    let data = sample_three_row_data();
    let indices = vec![2, 1, 0];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!("Bob"));
    assert_eq!(reordered[1][0], serde_json::json!("Alice"));
    assert_eq!(reordered[2][0], serde_json::json!("Charlie"));
}

#[test]
fn test_reorder_rows_empty_data() {
    let data = TomlTableData { headers: vec!["name".to_string()], rows: vec![] };
    let reordered = reorder_rows(&data, &[]);
    assert!(reordered.is_empty());
}

#[test]
fn test_reorder_rows_out_of_bounds_skipped() {
    let data = sample_three_row_data();
    let indices = vec![0, 99, 2];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered.len(), 2);
    assert_eq!(reordered[0][0], serde_json::json!("Charlie"));
    assert_eq!(reordered[1][0], serde_json::json!("Bob"));
}

#[test]
fn test_apply_sort_to_data_ascending() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));
}

#[test]
fn test_apply_sort_to_data_none_returns_unchanged() {
    let data = sample_three_row_data();
    let original_first = data.rows[0][0].clone();
    let result = apply_sort_to_data(data, None);
    assert_eq!(result.rows[0][0], original_first);
}

#[test]
fn test_apply_sort_to_data_preserves_headers() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.headers, vec!["name".to_string(), "cost".to_string()]);
}

#[test]
fn test_apply_sort_to_data_preserves_row_count() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows.len(), 3);
}

#[test]
fn test_apply_sort_to_data_with_mapping_ascending() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&sort_state));

    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));

    let mapping = mapping.expect("Expected mapping");
    assert_eq!(mapping, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_to_data_with_mapping_none_returns_no_mapping() {
    let data = sample_three_row_data();
    let (result, mapping) = apply_sort_to_data_with_mapping(data, None);
    assert_eq!(result.rows[0][0], serde_json::json!("Charlie"));
    assert!(mapping.is_none());
}

#[test]
fn test_apply_sort_to_data_with_mapping_preserves_headers() {
    let data = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());
    let (sorted, _) = apply_sort_to_data_with_mapping(data, Some(&sort_state));
    assert_eq!(sorted.headers, vec!["name".to_string(), "cost".to_string()]);
}

#[test]
fn test_apply_sort_to_data_with_mapping_consistent_with_apply_sort_to_data() {
    let data1 = sample_three_row_data();
    let data2 = sample_three_row_data();
    let sort_state = SortState::ascending("name".to_string());

    let sorted = apply_sort_to_data(data1, Some(&sort_state));
    let (sorted_with_mapping, _) = apply_sort_to_data_with_mapping(data2, Some(&sort_state));

    assert_eq!(sorted.headers, sorted_with_mapping.headers);
    assert_eq!(sorted.rows, sorted_with_mapping.rows);
}

#[test]
fn test_end_to_end_sort_and_translate() {
    let manager = SortStateManager::new();
    let data = sample_three_row_data();
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

#[test]
fn test_end_to_end_sort_descending_and_translate() {
    let manager = SortStateManager::new();
    let data = sample_three_row_data();
    let sort_state = SortState::descending("cost".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&sort_state));
    manager.set_row_mapping("/test.toml", "cards", mapping.unwrap());

    assert_eq!(sorted.rows[0][1], serde_json::json!(3));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 0);

    assert_eq!(sorted.rows[1][1], serde_json::json!(2));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 1), 2);

    assert_eq!(sorted.rows[2][1], serde_json::json!(1));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 2), 1);
}

#[test]
fn test_sort_case_insensitive_strings() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("banana")], vec![serde_json::json!("Apple")], vec![
            serde_json::json!("Cherry"),
        ]],
    };
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!("Apple"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("banana"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Cherry"));
}

#[test]
fn test_sort_many_rows() {
    let mut rows = Vec::new();
    for i in (0..100).rev() {
        rows.push(vec![serde_json::json!(i)]);
    }
    let data = TomlTableData { headers: vec!["value".to_string()], rows };

    let sort_state = SortState::ascending("value".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));

    for i in 0..100 {
        assert_eq!(sorted.rows[i][0], serde_json::json!(i as i64));
    }
}

#[test]
fn test_sort_multiple_null_values() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![
            vec![serde_json::json!(null)],
            vec![serde_json::json!("Alice")],
            vec![serde_json::json!(null)],
            vec![serde_json::json!("Bob")],
        ],
    };
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!(null));
    assert_eq!(sorted.rows[3][0], serde_json::json!(null));
}

#[test]
fn test_sort_float_precision() {
    let data = TomlTableData {
        headers: vec!["value".to_string()],
        rows: vec![vec![serde_json::json!(1.11)], vec![serde_json::json!(1.1)], vec![
            serde_json::json!(1.111),
        ]],
    };
    let sort_state = SortState::ascending("value".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!(1.1));
    assert_eq!(sorted.rows[1][0], serde_json::json!(1.11));
    assert_eq!(sorted.rows[2][0], serde_json::json!(1.111));
}

#[test]
fn test_sort_negative_numbers() {
    let data = TomlTableData {
        headers: vec!["value".to_string()],
        rows: vec![
            vec![serde_json::json!(5)],
            vec![serde_json::json!(-10)],
            vec![serde_json::json!(0)],
            vec![serde_json::json!(-3)],
        ],
    };
    let sort_state = SortState::ascending("value".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!(-10));
    assert_eq!(sorted.rows[1][0], serde_json::json!(-3));
    assert_eq!(sorted.rows[2][0], serde_json::json!(0));
    assert_eq!(sorted.rows[3][0], serde_json::json!(5));
}

#[test]
fn test_sort_preserves_multi_column_rows() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string(), "rarity".to_string()],
        rows: vec![
            vec![serde_json::json!("Charlie"), serde_json::json!(3), serde_json::json!("Common")],
            vec![serde_json::json!("Alice"), serde_json::json!(1), serde_json::json!("Rare")],
        ],
    };
    let sort_state = SortState::ascending("name".to_string());
    let sorted = apply_sort_to_data(data, Some(&sort_state));
    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[0][1], serde_json::json!(1));
    assert_eq!(sorted.rows[0][2], serde_json::json!("Rare"));
}
