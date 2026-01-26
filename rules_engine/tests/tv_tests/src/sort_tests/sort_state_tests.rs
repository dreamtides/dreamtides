use tv_lib::sort::sort_state::{
    apply_sort, apply_sort_to_data, apply_sort_to_data_with_mapping, reorder_rows, SortStateManager,
};
use tv_lib::sort::sort_types::{CellValue, SortDirection, SortState};
use tv_lib::toml::document_loader::TomlTableData;

fn make_table(headers: Vec<&str>, rows: Vec<Vec<serde_json::Value>>) -> TomlTableData {
    TomlTableData { headers: headers.into_iter().map(String::from).collect(), rows }
}

#[test]
fn test_sort_state_manager_initially_empty() {
    let manager = SortStateManager::new();
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());
}

#[test]
fn test_sort_state_manager_set_returns_previous() {
    let manager = SortStateManager::new();
    let state = SortState::ascending("name".to_string());

    let previous = manager.set_sort_state("/test.toml", "cards", Some(state.clone()));
    assert!(previous.is_none());

    let state2 = SortState::descending("cost".to_string());
    let previous = manager.set_sort_state("/test.toml", "cards", Some(state2));
    assert_eq!(previous, Some(state));
}

#[test]
fn test_sort_state_manager_set_none_removes() {
    let manager = SortStateManager::new();
    let state = SortState::ascending("name".to_string());
    manager.set_sort_state("/test.toml", "cards", Some(state.clone()));

    let previous = manager.set_sort_state("/test.toml", "cards", None);
    assert_eq!(previous, Some(state));
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
}

#[test]
fn test_sort_state_manager_table_name_isolation() {
    let manager = SortStateManager::new();
    let state_a = SortState::ascending("name".to_string());
    let state_b = SortState::descending("cost".to_string());

    manager.set_sort_state("/test.toml", "cards", Some(state_a.clone()));
    manager.set_sort_state("/test.toml", "effects", Some(state_b.clone()));

    assert_eq!(manager.get_sort_state("/test.toml", "cards"), Some(state_a));
    assert_eq!(manager.get_sort_state("/test.toml", "effects"), Some(state_b));
    assert!(manager.get_sort_state("/test.toml", "other").is_none());
}

#[test]
fn test_clear_sort_state_removes_state_and_mapping() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/test.toml", "cards", Some(SortState::ascending("name".to_string())));
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);

    manager.clear_sort_state("/test.toml", "cards");

    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());
}

#[test]
fn test_clear_sort_state_does_not_affect_other_tables() {
    let manager = SortStateManager::new();
    manager.set_sort_state("/test.toml", "cards", Some(SortState::ascending("name".to_string())));
    manager.set_sort_state(
        "/test.toml",
        "effects",
        Some(SortState::descending("cost".to_string())),
    );

    manager.clear_sort_state("/test.toml", "cards");

    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
    assert!(manager.get_sort_state("/test.toml", "effects").is_some());
}

#[test]
fn test_clear_nonexistent_sort_state_is_safe() {
    let manager = SortStateManager::new();
    manager.clear_sort_state("/nonexistent.toml", "cards");
}

#[test]
fn test_row_mapping_overwrite() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 0, 1]);
    manager.set_row_mapping("/test.toml", "cards", vec![1, 2, 0]);

    assert_eq!(manager.get_row_mapping("/test.toml", "cards"), Some(vec![1, 2, 0]));
}

#[test]
fn test_display_to_original_with_identity_mapping() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![0, 1, 2]);

    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 0);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 1), 1);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 2), 2);
}

#[test]
fn test_display_to_original_with_reversed_mapping() {
    let manager = SortStateManager::new();
    manager.set_row_mapping("/test.toml", "cards", vec![2, 1, 0]);

    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 2);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 1), 1);
    assert_eq!(manager.display_to_original("/test.toml", "cards", 2), 0);
}

#[test]
fn test_apply_sort_ascending_strings() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("Cherry")],
        vec![serde_json::json!("Apple")],
        vec![serde_json::json!("Banana")],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_descending_strings() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("Cherry")],
        vec![serde_json::json!("Apple")],
        vec![serde_json::json!("Banana")],
    ]);

    let state = SortState::descending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0, 2, 1]);
}

#[test]
fn test_apply_sort_ascending_integers() {
    let data = make_table(vec!["cost"], vec![
        vec![serde_json::json!(5)],
        vec![serde_json::json!(1)],
        vec![serde_json::json!(3)],
    ]);

    let state = SortState::ascending("cost".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_descending_integers() {
    let data = make_table(vec!["cost"], vec![
        vec![serde_json::json!(5)],
        vec![serde_json::json!(1)],
        vec![serde_json::json!(3)],
    ]);

    let state = SortState::descending("cost".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0, 2, 1]);
}

#[test]
fn test_apply_sort_floats() {
    let data = make_table(vec!["score"], vec![
        vec![serde_json::json!(3.14)],
        vec![serde_json::json!(1.0)],
        vec![serde_json::json!(2.72)],
    ]);

    let state = SortState::ascending("score".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_mixed_integer_and_float() {
    let data = make_table(vec!["value"], vec![
        vec![serde_json::json!(3)],
        vec![serde_json::json!(1.5)],
        vec![serde_json::json!(2)],
    ]);

    let state = SortState::ascending("value".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_booleans() {
    let data = make_table(vec!["active"], vec![
        vec![serde_json::json!(true)],
        vec![serde_json::json!(false)],
        vec![serde_json::json!(true)],
    ]);

    let state = SortState::ascending("active".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices[0], 1);
}

#[test]
fn test_apply_sort_nulls_sort_last_ascending() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!(null)],
        vec![serde_json::json!("Alice")],
        vec![serde_json::json!(null)],
        vec![serde_json::json!("Bob")],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices[0], 1);
    assert_eq!(indices[1], 3);
}

#[test]
fn test_apply_sort_nulls_sort_first_descending() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!(null)],
        vec![serde_json::json!("Alice")],
        vec![serde_json::json!("Bob")],
    ]);

    let state = SortState::descending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices[0], 0);
}

#[test]
fn test_apply_sort_case_insensitive_strings() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("banana")],
        vec![serde_json::json!("Apple")],
        vec![serde_json::json!("CHERRY")],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!("Apple"));
    assert_eq!(reordered[1][0], serde_json::json!("banana"));
    assert_eq!(reordered[2][0], serde_json::json!("CHERRY"));
}

#[test]
fn test_apply_sort_empty_table() {
    let data = make_table(vec!["name"], vec![]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert!(indices.is_empty());
}

#[test]
fn test_apply_sort_single_row() {
    let data = make_table(vec!["name"], vec![vec![serde_json::json!("Alice")]]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0]);
}

#[test]
fn test_apply_sort_nonexistent_column_returns_identity() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("Charlie")],
        vec![serde_json::json!("Alice")],
        vec![serde_json::json!("Bob")],
    ]);

    let state = SortState::ascending("nonexistent".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_apply_sort_stable_order_for_equal_values() {
    let data = make_table(vec!["priority", "name"], vec![
        vec![serde_json::json!(1), serde_json::json!("First")],
        vec![serde_json::json!(1), serde_json::json!("Second")],
        vec![serde_json::json!(1), serde_json::json!("Third")],
    ]);

    let state = SortState::ascending("priority".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_apply_sort_mixed_types_type_ordering() {
    let data = make_table(vec!["value"], vec![
        vec![serde_json::json!("text")],
        vec![serde_json::json!(42)],
        vec![serde_json::json!(true)],
        vec![serde_json::json!(null)],
    ]);

    let state = SortState::ascending("value".to_string());
    let indices = apply_sort(&data, &state);
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!(true));
    assert_eq!(reordered[1][0], serde_json::json!(42));
    assert_eq!(reordered[2][0], serde_json::json!("text"));
    assert_eq!(reordered[3][0], serde_json::json!(null));
}

#[test]
fn test_apply_sort_preserves_multi_column_row_data() {
    let data = make_table(vec!["name", "cost", "rarity"], vec![
        vec![serde_json::json!("Zap"), serde_json::json!(3), serde_json::json!("Common")],
        vec![serde_json::json!("Arc"), serde_json::json!(5), serde_json::json!("Rare")],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!("Arc"));
    assert_eq!(reordered[0][1], serde_json::json!(5));
    assert_eq!(reordered[0][2], serde_json::json!("Rare"));
    assert_eq!(reordered[1][0], serde_json::json!("Zap"));
    assert_eq!(reordered[1][1], serde_json::json!(3));
    assert_eq!(reordered[1][2], serde_json::json!("Common"));
}

#[test]
fn test_reorder_rows_skips_out_of_bounds_indices() {
    let data =
        make_table(vec!["name"], vec![vec![serde_json::json!("Alice")], vec![serde_json::json!(
            "Bob"
        )]]);

    let indices = vec![1, 99, 0];
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered.len(), 2);
    assert_eq!(reordered[0][0], serde_json::json!("Bob"));
    assert_eq!(reordered[1][0], serde_json::json!("Alice"));
}

#[test]
fn test_reorder_rows_empty_indices() {
    let data = make_table(vec!["name"], vec![vec![serde_json::json!("Alice")]]);

    let reordered = reorder_rows(&data, &[]);
    assert!(reordered.is_empty());
}

#[test]
fn test_apply_sort_to_data_with_mapping_returns_sorted_data_and_indices() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("Charlie")],
        vec![serde_json::json!("Alice")],
        vec![serde_json::json!("Bob")],
    ]);

    let state = SortState::ascending("name".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&state));

    assert_eq!(sorted.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Bob"));
    assert_eq!(sorted.rows[2][0], serde_json::json!("Charlie"));

    let mapping = mapping.expect("Expected mapping");
    assert_eq!(mapping, vec![1, 2, 0]);
}

#[test]
fn test_apply_sort_to_data_with_mapping_none_passthrough() {
    let data = make_table(vec!["name"], vec![vec![serde_json::json!("Charlie")], vec![
        serde_json::json!("Alice"),
    ]]);

    let (result, mapping) = apply_sort_to_data_with_mapping(data, None);
    assert_eq!(result.rows[0][0], serde_json::json!("Charlie"));
    assert_eq!(result.rows[1][0], serde_json::json!("Alice"));
    assert!(mapping.is_none());
}

#[test]
fn test_apply_sort_to_data_sorts_rows() {
    let data = make_table(vec!["cost"], vec![
        vec![serde_json::json!(10)],
        vec![serde_json::json!(2)],
        vec![serde_json::json!(7)],
    ]);

    let state = SortState::descending("cost".to_string());
    let sorted = apply_sort_to_data(data, Some(&state));
    assert_eq!(sorted.rows[0][0], serde_json::json!(10));
    assert_eq!(sorted.rows[1][0], serde_json::json!(7));
    assert_eq!(sorted.rows[2][0], serde_json::json!(2));
}

#[test]
fn test_apply_sort_to_data_none_is_identity() {
    let data = make_table(vec!["name"], vec![vec![serde_json::json!("Charlie")], vec![
        serde_json::json!("Alice"),
    ]]);

    let sorted = apply_sort_to_data(data, None);
    assert_eq!(sorted.rows[0][0], serde_json::json!("Charlie"));
    assert_eq!(sorted.rows[1][0], serde_json::json!("Alice"));
}

#[test]
fn test_cell_value_from_json_null() {
    let v = CellValue::from_json(&serde_json::json!(null));
    assert_eq!(v, CellValue::Null);
}

#[test]
fn test_cell_value_from_json_bool() {
    let v = CellValue::from_json(&serde_json::json!(true));
    assert_eq!(v, CellValue::Boolean(true));
}

#[test]
fn test_cell_value_from_json_integer() {
    let v = CellValue::from_json(&serde_json::json!(42));
    assert_eq!(v, CellValue::Integer(42));
}

#[test]
fn test_cell_value_from_json_float() {
    let v = CellValue::from_json(&serde_json::json!(3.14));
    assert_eq!(v, CellValue::Float(3.14));
}

#[test]
fn test_cell_value_from_json_string() {
    let v = CellValue::from_json(&serde_json::json!("hello"));
    assert_eq!(v, CellValue::String("hello".to_string()));
}

#[test]
fn test_cell_value_from_json_array() {
    let v = CellValue::from_json(&serde_json::json!([1, 2, 3]));
    assert!(matches!(v, CellValue::String(_)));
}

#[test]
fn test_cell_value_from_json_object() {
    let v = CellValue::from_json(&serde_json::json!({"key": "value"}));
    assert!(matches!(v, CellValue::String(_)));
}

#[test]
fn test_cell_value_ordering_null_vs_non_null() {
    let null = CellValue::Null;
    let number = CellValue::Integer(5);
    assert!(null > number);
}

#[test]
fn test_cell_value_ordering_null_vs_null() {
    let a = CellValue::Null;
    let b = CellValue::Null;
    assert_eq!(a.cmp_values(&b), std::cmp::Ordering::Equal);
}

#[test]
fn test_cell_value_ordering_integer_vs_float() {
    let i = CellValue::Integer(3);
    let f = CellValue::Float(3.0);
    assert_eq!(i.cmp_values(&f), std::cmp::Ordering::Equal);

    let i2 = CellValue::Integer(5);
    let f2 = CellValue::Float(3.7);
    assert!(i2 > f2);
}

#[test]
fn test_cell_value_ordering_float_vs_integer() {
    let f = CellValue::Float(5.5);
    let i = CellValue::Integer(5);
    assert!(f > i);
}

#[test]
fn test_cell_value_ordering_negative_numbers() {
    let a = CellValue::Integer(-10);
    let b = CellValue::Integer(5);
    assert!(a < b);
}

#[test]
fn test_cell_value_ordering_string_case_insensitive() {
    let a = CellValue::String("apple".to_string());
    let b = CellValue::String("Banana".to_string());
    assert!(a < b);
}

#[test]
fn test_cell_value_ordering_bool_vs_number() {
    let b = CellValue::Boolean(true);
    let n = CellValue::Integer(5);
    assert!(b < n);
}

#[test]
fn test_cell_value_ordering_number_vs_string() {
    let n = CellValue::Integer(42);
    let s = CellValue::String("text".to_string());
    assert!(n < s);
}

#[test]
fn test_sort_direction_toggle() {
    assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
    assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
}

#[test]
fn test_sort_direction_default_is_ascending() {
    let dir = SortDirection::default();
    assert_eq!(dir, SortDirection::Ascending);
}

#[test]
fn test_sort_state_constructors() {
    let asc = SortState::ascending("name".to_string());
    assert_eq!(asc.column, "name");
    assert_eq!(asc.direction, SortDirection::Ascending);

    let desc = SortState::descending("cost".to_string());
    assert_eq!(desc.column, "cost");
    assert_eq!(desc.direction, SortDirection::Descending);
}

#[test]
fn test_sort_state_new() {
    let state = SortState::new("col".to_string(), SortDirection::Descending);
    assert_eq!(state.column, "col");
    assert_eq!(state.direction, SortDirection::Descending);
}

#[test]
fn test_sort_state_clone_and_eq() {
    let state = SortState::ascending("name".to_string());
    let clone = state.clone();
    assert_eq!(state, clone);
}

#[test]
fn test_sort_state_debug() {
    let state = SortState::ascending("name".to_string());
    let debug = format!("{state:?}");
    assert!(debug.contains("name"));
    assert!(debug.contains("Ascending"));
}

#[test]
fn test_sort_direction_serialization() {
    let asc = SortDirection::Ascending;
    let json = serde_json::to_value(asc).unwrap();
    assert_eq!(json, serde_json::json!("ascending"));

    let desc = SortDirection::Descending;
    let json = serde_json::to_value(desc).unwrap();
    assert_eq!(json, serde_json::json!("descending"));
}

#[test]
fn test_sort_direction_deserialization() {
    let asc: SortDirection = serde_json::from_value(serde_json::json!("ascending")).unwrap();
    assert_eq!(asc, SortDirection::Ascending);

    let desc: SortDirection = serde_json::from_value(serde_json::json!("descending")).unwrap();
    assert_eq!(desc, SortDirection::Descending);
}

#[test]
fn test_sort_state_serialization_roundtrip() {
    let state = SortState::ascending("name".to_string());
    let json = serde_json::to_value(&state).unwrap();
    let deserialized: SortState = serde_json::from_value(json).unwrap();
    assert_eq!(state, deserialized);
}

#[test]
fn test_end_to_end_sort_and_translate() {
    let manager = SortStateManager::new();

    let data = make_table(vec!["name", "cost"], vec![
        vec![serde_json::json!("Fireball"), serde_json::json!(5)],
        vec![serde_json::json!("Arcane Shield"), serde_json::json!(2)],
        vec![serde_json::json!("Dragon"), serde_json::json!(8)],
    ]);

    let state = SortState::ascending("name".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data, Some(&state));
    let mapping = mapping.unwrap();

    manager.set_sort_state("/cards.toml", "cards", Some(state));
    manager.set_row_mapping("/cards.toml", "cards", mapping);

    assert_eq!(sorted.rows[0][0], serde_json::json!("Arcane Shield"));
    assert_eq!(manager.display_to_original("/cards.toml", "cards", 0), 1);

    assert_eq!(sorted.rows[1][0], serde_json::json!("Dragon"));
    assert_eq!(manager.display_to_original("/cards.toml", "cards", 1), 2);

    assert_eq!(sorted.rows[2][0], serde_json::json!("Fireball"));
    assert_eq!(manager.display_to_original("/cards.toml", "cards", 2), 0);
}

#[test]
fn test_end_to_end_sort_clear_and_resort() {
    let manager = SortStateManager::new();

    let data1 = make_table(vec!["name", "cost"], vec![
        vec![serde_json::json!("Charlie"), serde_json::json!(3)],
        vec![serde_json::json!("Alice"), serde_json::json!(1)],
        vec![serde_json::json!("Bob"), serde_json::json!(2)],
    ]);

    let name_sort = SortState::ascending("name".to_string());
    let (_, mapping) = apply_sort_to_data_with_mapping(data1, Some(&name_sort));
    manager.set_sort_state("/test.toml", "cards", Some(name_sort));
    manager.set_row_mapping("/test.toml", "cards", mapping.unwrap());

    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 1);

    manager.clear_sort_state("/test.toml", "cards");
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
    assert!(manager.get_row_mapping("/test.toml", "cards").is_none());

    let data2 = make_table(vec!["name", "cost"], vec![
        vec![serde_json::json!("Charlie"), serde_json::json!(3)],
        vec![serde_json::json!("Alice"), serde_json::json!(1)],
        vec![serde_json::json!("Bob"), serde_json::json!(2)],
    ]);

    let cost_sort = SortState::ascending("cost".to_string());
    let (sorted, mapping) = apply_sort_to_data_with_mapping(data2, Some(&cost_sort));
    manager.set_sort_state("/test.toml", "cards", Some(cost_sort));
    manager.set_row_mapping("/test.toml", "cards", mapping.unwrap());

    assert_eq!(sorted.rows[0][1], serde_json::json!(1));
    assert_eq!(manager.display_to_original("/test.toml", "cards", 0), 1);
}

#[test]
fn test_apply_sort_large_dataset() {
    let rows: Vec<Vec<serde_json::Value>> =
        (0..100).rev().map(|i| vec![serde_json::json!(i)]).collect();
    let data = make_table(vec!["id"], rows);

    let state = SortState::ascending("id".to_string());
    let indices = apply_sort(&data, &state);

    for (display_idx, &original_idx) in indices.iter().enumerate() {
        let value = data.rows[original_idx][0].as_i64().unwrap();
        assert_eq!(value, display_idx as i64);
    }
}

#[test]
fn test_apply_sort_all_nulls() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!(null)],
        vec![serde_json::json!(null)],
        vec![serde_json::json!(null)],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_apply_sort_duplicate_values() {
    let data = make_table(vec!["grade"], vec![
        vec![serde_json::json!("B")],
        vec![serde_json::json!("A")],
        vec![serde_json::json!("B")],
        vec![serde_json::json!("A")],
    ]);

    let state = SortState::ascending("grade".to_string());
    let indices = apply_sort(&data, &state);
    assert_eq!(indices[0], 1);
    assert_eq!(indices[1], 3);
    assert_eq!(indices[2], 0);
    assert_eq!(indices[3], 2);
}

#[test]
fn test_apply_sort_empty_strings() {
    let data = make_table(vec!["name"], vec![
        vec![serde_json::json!("Bob")],
        vec![serde_json::json!("")],
        vec![serde_json::json!("Alice")],
    ]);

    let state = SortState::ascending("name".to_string());
    let indices = apply_sort(&data, &state);
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!(""));
    assert_eq!(reordered[1][0], serde_json::json!("Alice"));
    assert_eq!(reordered[2][0], serde_json::json!("Bob"));
}

#[test]
fn test_sort_state_manager_default_trait() {
    let manager = SortStateManager::default();
    assert!(manager.get_sort_state("/test.toml", "cards").is_none());
}

#[test]
fn test_apply_sort_negative_and_positive_numbers() {
    let data = make_table(vec!["temp"], vec![
        vec![serde_json::json!(10)],
        vec![serde_json::json!(-5)],
        vec![serde_json::json!(0)],
        vec![serde_json::json!(-20)],
    ]);

    let state = SortState::ascending("temp".to_string());
    let indices = apply_sort(&data, &state);
    let reordered = reorder_rows(&data, &indices);
    assert_eq!(reordered[0][0], serde_json::json!(-20));
    assert_eq!(reordered[1][0], serde_json::json!(-5));
    assert_eq!(reordered[2][0], serde_json::json!(0));
    assert_eq!(reordered[3][0], serde_json::json!(10));
}
