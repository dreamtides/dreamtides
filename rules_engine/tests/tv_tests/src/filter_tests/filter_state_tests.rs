use tv_lib::filter::filter_state::{
    apply_filter, apply_filter_to_data, apply_filter_to_data_with_visibility, compute_visibility,
    visible_row_indices, FilterStateManager,
};
use tv_lib::filter::filter_types::{matches_condition, FilterState};
use tv_lib::sort::sort_types::CellValue;
use tv_lib::toml::document_loader::TomlTableData;
use tv_lib::toml::metadata_types::{ColumnFilter, FilterCondition};

#[test]
fn test_matches_contains_string() {
    let cell = CellValue::String("Hello World".to_string());
    let cond = FilterCondition::Contains("world".to_string());
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_case_insensitive() {
    let cell = CellValue::String("FooBar".to_string());
    let cond = FilterCondition::Contains("foobar".to_string());
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_no_match() {
    let cell = CellValue::String("Hello".to_string());
    let cond = FilterCondition::Contains("xyz".to_string());
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_integer() {
    let cell = CellValue::Integer(42);
    let cond = FilterCondition::Contains("42".to_string());
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_float() {
    let cell = CellValue::Float(3.14);
    let cond = FilterCondition::Contains("3.14".to_string());
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_boolean() {
    let cell = CellValue::Boolean(true);
    let cond = FilterCondition::Contains("true".to_string());
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_contains_null() {
    let cell = CellValue::Null;
    let cond = FilterCondition::Contains("anything".to_string());
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_string() {
    let cell = CellValue::String("hello".to_string());
    let cond = FilterCondition::Equals(serde_json::json!("Hello"));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_string_no_match() {
    let cell = CellValue::String("hello".to_string());
    let cond = FilterCondition::Equals(serde_json::json!("world"));
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_integer() {
    let cell = CellValue::Integer(42);
    let cond = FilterCondition::Equals(serde_json::json!(42));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_float() {
    let cell = CellValue::Float(3.14);
    let cond = FilterCondition::Equals(serde_json::json!(3.14));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_boolean() {
    let cell = CellValue::Boolean(true);
    let cond = FilterCondition::Equals(serde_json::json!(true));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_null() {
    let cell = CellValue::Null;
    let cond = FilterCondition::Equals(serde_json::json!(null));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_type_mismatch() {
    let cell = CellValue::String("42".to_string());
    let cond = FilterCondition::Equals(serde_json::json!(42));
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_within() {
    let cell = CellValue::Integer(5);
    let cond = FilterCondition::Range { min: Some(1.0), max: Some(10.0) };
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_below_min() {
    let cell = CellValue::Integer(0);
    let cond = FilterCondition::Range { min: Some(1.0), max: Some(10.0) };
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_above_max() {
    let cell = CellValue::Integer(11);
    let cond = FilterCondition::Range { min: Some(1.0), max: Some(10.0) };
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_min_only() {
    let cell = CellValue::Float(100.0);
    let cond = FilterCondition::Range { min: Some(5.0), max: None };
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_max_only() {
    let cell = CellValue::Float(3.0);
    let cond = FilterCondition::Range { min: None, max: Some(5.0) };
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_on_boundary() {
    let cell = CellValue::Integer(10);
    let cond = FilterCondition::Range { min: Some(10.0), max: Some(10.0) };
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_non_numeric() {
    let cell = CellValue::String("not a number".to_string());
    let cond = FilterCondition::Range { min: Some(1.0), max: Some(10.0) };
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_range_null() {
    let cell = CellValue::Null;
    let cond = FilterCondition::Range { min: Some(1.0), max: Some(10.0) };
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_boolean_true() {
    let cell = CellValue::Boolean(true);
    let cond = FilterCondition::Boolean(true);
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_boolean_false() {
    let cell = CellValue::Boolean(false);
    let cond = FilterCondition::Boolean(false);
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_boolean_mismatch() {
    let cell = CellValue::Boolean(true);
    let cond = FilterCondition::Boolean(false);
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_matches_boolean_non_boolean_cell() {
    let cell = CellValue::String("true".to_string());
    let cond = FilterCondition::Boolean(true);
    assert!(!matches_condition(&cell, &cond));
}

#[test]
fn test_compute_visibility_no_filters() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let visibility = compute_visibility(&data, &[]);
    assert_eq!(visibility, vec![true, true]);
}

#[test]
fn test_compute_visibility_single_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(true)],
            vec![serde_json::json!("Bob"), serde_json::json!(false)],
            vec![serde_json::json!("Charlie"), serde_json::json!(true)],
        ],
    };

    let filters = vec![ColumnFilter::new("active", FilterCondition::Boolean(true))];
    let visibility = compute_visibility(&data, &filters);
    assert_eq!(visibility, vec![true, false, true]);
}

#[test]
fn test_compute_visibility_multiple_filters_and_logic() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "age".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(25), serde_json::json!(true)],
            vec![serde_json::json!("Bob"), serde_json::json!(30), serde_json::json!(false)],
            vec![serde_json::json!("Charlie"), serde_json::json!(35), serde_json::json!(true)],
            vec![serde_json::json!("Diana"), serde_json::json!(20), serde_json::json!(true)],
        ],
    };

    let filters = vec![
        ColumnFilter::new("active", FilterCondition::Boolean(true)),
        ColumnFilter::new("age", FilterCondition::Range { min: Some(25.0), max: None }),
    ];
    let visibility = compute_visibility(&data, &filters);
    assert_eq!(visibility, vec![true, false, true, false]);
}

#[test]
fn test_compute_visibility_nonexistent_column_passes() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let filters =
        vec![ColumnFilter::new("nonexistent", FilterCondition::Contains("x".to_string()))];
    let visibility = compute_visibility(&data, &filters);
    assert_eq!(visibility, vec![true, true]);
}

#[test]
fn test_visible_row_indices() {
    let visibility = vec![true, false, true, false, true];
    assert_eq!(visible_row_indices(&visibility), vec![0, 2, 4]);
}

#[test]
fn test_visible_row_indices_all_visible() {
    let visibility = vec![true, true, true];
    assert_eq!(visible_row_indices(&visibility), vec![0, 1, 2]);
}

#[test]
fn test_visible_row_indices_none_visible() {
    let visibility = vec![false, false, false];
    assert!(visible_row_indices(&visibility).is_empty());
}

#[test]
fn test_apply_filter_returns_visible_indices() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "score".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(80)],
            vec![serde_json::json!("Bob"), serde_json::json!(40)],
            vec![serde_json::json!("Charlie"), serde_json::json!(90)],
        ],
    };

    let filters =
        vec![ColumnFilter::new("score", FilterCondition::Range { min: Some(50.0), max: None })];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![0, 2]);
}

#[test]
fn test_apply_filter_to_data_with_visibility_active() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(true)],
            vec![serde_json::json!("Bob"), serde_json::json!(false)],
            vec![serde_json::json!("Charlie"), serde_json::json!(true)],
        ],
    };

    let filter_state =
        FilterState::active(vec![ColumnFilter::new("active", FilterCondition::Boolean(true))]);

    let (filtered, visibility) = apply_filter_to_data_with_visibility(data, Some(&filter_state));
    assert_eq!(filtered.rows.len(), 2);
    assert_eq!(filtered.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(filtered.rows[1][0], serde_json::json!("Charlie"));

    let vis = visibility.expect("Expected visibility vector");
    assert_eq!(vis, vec![true, false, true]);
}

#[test]
fn test_apply_filter_to_data_with_visibility_inactive() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let filter_state = FilterState::new(
        vec![ColumnFilter::new("name", FilterCondition::Contains("Alice".to_string()))],
        false,
    );

    let (result, visibility) = apply_filter_to_data_with_visibility(data, Some(&filter_state));
    assert_eq!(result.rows.len(), 2);
    assert!(visibility.is_none());
}

#[test]
fn test_apply_filter_to_data_with_visibility_none() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let (result, visibility) = apply_filter_to_data_with_visibility(data, None);
    assert_eq!(result.rows.len(), 2);
    assert!(visibility.is_none());
}

#[test]
fn test_apply_filter_to_data_returns_filtered_rows() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "score".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(80)],
            vec![serde_json::json!("Bob"), serde_json::json!(40)],
            vec![serde_json::json!("Charlie"), serde_json::json!(90)],
        ],
    };

    let filter_state =
        FilterState::active(vec![ColumnFilter::new("score", FilterCondition::Range {
            min: Some(50.0),
            max: None,
        })]);

    let result = apply_filter_to_data(data, Some(&filter_state));
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0][0], serde_json::json!("Alice"));
    assert_eq!(result.rows[1][0], serde_json::json!("Charlie"));
}

#[test]
fn test_filter_state_manager_get_set() {
    let manager = FilterStateManager::new();
    assert!(manager.get_filter_state("/test.toml", "cards").is_none());

    let state = FilterState::active(vec![ColumnFilter::new(
        "name",
        FilterCondition::Contains("test".to_string()),
    )]);
    manager.set_filter_state("/test.toml", "cards", Some(state.clone()));

    let retrieved = manager.get_filter_state("/test.toml", "cards");
    assert_eq!(retrieved, Some(state));
}

#[test]
fn test_filter_state_manager_per_file_isolation() {
    let manager = FilterStateManager::new();
    let state_a = FilterState::active(vec![ColumnFilter::new(
        "name",
        FilterCondition::Contains("a".to_string()),
    )]);
    let state_b = FilterState::active(vec![ColumnFilter::new(
        "name",
        FilterCondition::Contains("b".to_string()),
    )]);

    manager.set_filter_state("/file_a.toml", "cards", Some(state_a.clone()));
    manager.set_filter_state("/file_b.toml", "cards", Some(state_b.clone()));

    assert_eq!(manager.get_filter_state("/file_a.toml", "cards"), Some(state_a));
    assert_eq!(manager.get_filter_state("/file_b.toml", "cards"), Some(state_b));
    assert!(manager.get_filter_state("/file_c.toml", "cards").is_none());
}

#[test]
fn test_filter_state_manager_clear() {
    let manager = FilterStateManager::new();
    let state = FilterState::active(vec![ColumnFilter::new(
        "name",
        FilterCondition::Contains("test".to_string()),
    )]);

    manager.set_filter_state("/test.toml", "cards", Some(state));
    manager.set_visibility("/test.toml", "cards", vec![true, false, true]);

    manager.clear_filter_state("/test.toml", "cards");
    assert!(manager.get_filter_state("/test.toml", "cards").is_none());
    assert!(manager.get_visibility("/test.toml", "cards").is_none());
}

#[test]
fn test_filter_state_manager_visibility() {
    let manager = FilterStateManager::new();
    assert!(manager.get_visibility("/test.toml", "cards").is_none());

    manager.set_visibility("/test.toml", "cards", vec![true, false, true]);
    assert_eq!(manager.get_visibility("/test.toml", "cards"), Some(vec![true, false, true]));
}

#[test]
fn test_filter_state_manager_is_row_visible() {
    let manager = FilterStateManager::new();

    assert!(manager.is_row_visible("/test.toml", "cards", 0));

    manager.set_visibility("/test.toml", "cards", vec![true, false, true]);
    assert!(manager.is_row_visible("/test.toml", "cards", 0));
    assert!(!manager.is_row_visible("/test.toml", "cards", 1));
    assert!(manager.is_row_visible("/test.toml", "cards", 2));
}

#[test]
fn test_filter_state_manager_is_row_visible_out_of_bounds() {
    let manager = FilterStateManager::new();
    manager.set_visibility("/test.toml", "cards", vec![true, false]);

    assert!(manager.is_row_visible("/test.toml", "cards", 10));
}

#[test]
fn test_text_substring_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "description".to_string()],
        rows: vec![
            vec![serde_json::json!("Fireball"), serde_json::json!("Deals fire damage")],
            vec![serde_json::json!("Ice Shield"), serde_json::json!("Blocks ice attacks")],
            vec![serde_json::json!("Fire Storm"), serde_json::json!("Massive fire area")],
        ],
    };

    let filters = vec![ColumnFilter::new("name", FilterCondition::Contains("fire".to_string()))];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![0, 2]);
}

#[test]
fn test_exact_value_filter_enum() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "rarity".to_string()],
        rows: vec![
            vec![serde_json::json!("Card A"), serde_json::json!("Common")],
            vec![serde_json::json!("Card B"), serde_json::json!("Rare")],
            vec![serde_json::json!("Card C"), serde_json::json!("Common")],
        ],
    };

    let filters =
        vec![ColumnFilter::new("rarity", FilterCondition::Equals(serde_json::json!("Common")))];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![0, 2]);
}

#[test]
fn test_numeric_range_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Cheap"), serde_json::json!(1)],
            vec![serde_json::json!("Medium"), serde_json::json!(5)],
            vec![serde_json::json!("Expensive"), serde_json::json!(10)],
            vec![serde_json::json!("Very Expensive"), serde_json::json!(20)],
        ],
    };

    let filters =
        vec![ColumnFilter::new("cost", FilterCondition::Range { min: Some(3.0), max: Some(15.0) })];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![1, 2]);
}

#[test]
fn test_boolean_checkbox_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "completed".to_string()],
        rows: vec![
            vec![serde_json::json!("Task 1"), serde_json::json!(true)],
            vec![serde_json::json!("Task 2"), serde_json::json!(false)],
            vec![serde_json::json!("Task 3"), serde_json::json!(true)],
            vec![serde_json::json!("Task 4"), serde_json::json!(false)],
        ],
    };

    let filters = vec![ColumnFilter::new("completed", FilterCondition::Boolean(false))];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![1, 3]);
}

#[test]
fn test_multiple_filters_combined_and() {
    let data = TomlTableData {
        headers: vec![
            "name".to_string(),
            "cost".to_string(),
            "rarity".to_string(),
            "active".to_string(),
        ],
        rows: vec![
            vec![
                serde_json::json!("Card A"),
                serde_json::json!(3),
                serde_json::json!("Common"),
                serde_json::json!(true),
            ],
            vec![
                serde_json::json!("Card B"),
                serde_json::json!(7),
                serde_json::json!("Rare"),
                serde_json::json!(true),
            ],
            vec![
                serde_json::json!("Card C"),
                serde_json::json!(5),
                serde_json::json!("Common"),
                serde_json::json!(false),
            ],
            vec![
                serde_json::json!("Card D"),
                serde_json::json!(8),
                serde_json::json!("Common"),
                serde_json::json!(true),
            ],
        ],
    };

    let filters = vec![
        ColumnFilter::new("rarity", FilterCondition::Equals(serde_json::json!("Common"))),
        ColumnFilter::new("active", FilterCondition::Boolean(true)),
        ColumnFilter::new("cost", FilterCondition::Range { min: Some(1.0), max: Some(5.0) }),
    ];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![0]);
}

#[test]
fn test_filter_empty_data() {
    let data = TomlTableData { headers: vec!["name".to_string()], rows: Vec::new() };

    let filters = vec![ColumnFilter::new("name", FilterCondition::Contains("test".to_string()))];
    let indices = apply_filter(&data, &filters);
    assert!(indices.is_empty());
}

#[test]
fn test_filter_with_null_values() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "score".to_string()],
        rows: vec![
            vec![serde_json::json!("Alice"), serde_json::json!(80)],
            vec![serde_json::json!("Bob"), serde_json::json!(null)],
            vec![serde_json::json!("Charlie"), serde_json::json!(90)],
        ],
    };

    let filters =
        vec![ColumnFilter::new("score", FilterCondition::Range { min: Some(50.0), max: None })];
    let indices = apply_filter(&data, &filters);
    assert_eq!(indices, vec![0, 2]);
}

#[test]
fn test_matches_equals_integer_to_float() {
    let cell = CellValue::Integer(5);
    let cond = FilterCondition::Equals(serde_json::json!(5.0));
    assert!(matches_condition(&cell, &cond));
}

#[test]
fn test_matches_equals_float_to_integer() {
    let cell = CellValue::Float(5.0);
    let cond = FilterCondition::Equals(serde_json::json!(5));
    assert!(matches_condition(&cell, &cond));
}
