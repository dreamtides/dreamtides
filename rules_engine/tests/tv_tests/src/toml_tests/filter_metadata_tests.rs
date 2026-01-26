use tv_lib::filter::filter_state::{compute_hidden_rows, compute_visible_rows, FilterStateManager};
use tv_lib::filter::filter_types::{ColumnFilterState, FilterConditionState};
use tv_lib::toml::document_loader::TomlTableData;
use tv_lib::toml::metadata_parser::parse_filter_config_from_content;
use tv_lib::toml::metadata_serializer::update_filter_config_with_fs;
use tv_lib::toml::metadata_types::{ColumnFilter, FilterCondition, FilterConfig};

use crate::test_utils::mock_filesystem::MockFileSystem;

#[test]
fn test_parse_filter_config_with_contains() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "name"
condition = { contains = "fire" }
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert!(config.active);
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.filters[0].column, "name");
    assert!(matches!(config.filters[0].condition, FilterCondition::Contains(ref s) if s == "fire"));
}

#[test]
fn test_parse_filter_config_with_equals() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "rarity"
condition = { equals = "rare" }
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert_eq!(config.filters.len(), 1);
    assert!(matches!(config.filters[0].condition, FilterCondition::Equals(ref v) if v == "rare"));
}

#[test]
fn test_parse_filter_config_with_range() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "cost"
condition = { min = 1, max = 5 }
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert_eq!(config.filters.len(), 1);
    assert!(matches!(
        config.filters[0].condition,
        FilterCondition::Range { min: Some(m1), max: Some(m2) } if (m1 - 1.0).abs() < f64::EPSILON && (m2 - 5.0).abs() < f64::EPSILON
    ));
}

#[test]
fn test_parse_filter_config_with_boolean() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "active"
condition = { boolean = true }
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert_eq!(config.filters.len(), 1);
    assert!(matches!(config.filters[0].condition, FilterCondition::Boolean(true)));
}

#[test]
fn test_parse_filter_config_missing_metadata() {
    let content = r#"
[[cards]]
id = "card-1"
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_filter_config_missing_filter_section() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_filter_config_inactive() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = false
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert!(!config.active);
    assert!(config.filters.is_empty());
}

#[test]
fn test_parse_filter_config_multiple_filters() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "name"
condition = { contains = "fire" }

[[metadata.filter.filters]]
column = "cost"
condition = { min = 2, max = 8 }
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config to be present");
    assert_eq!(config.filters.len(), 2);
    assert_eq!(config.filters[0].column, "name");
    assert_eq!(config.filters[1].column, "cost");
}

#[test]
fn test_update_filter_config_adds_filter_section() {
    let fs = MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
"#,
    );
    let filter_config = FilterConfig {
        filters: vec![ColumnFilter::new("name", FilterCondition::Contains("fire".to_string()))],
        active: true,
    };

    let result = update_filter_config_with_fs(&fs, "/test.toml", Some(&filter_config));
    assert!(result.is_ok());

    let saved = fs.last_written_content().unwrap();
    assert!(saved.contains("[metadata]"), "Expected [metadata] section in:\n{saved}");
    assert!(saved.contains("active = true"), "Expected active = true in:\n{saved}");
    assert!(saved.contains("column = \"name\""), "Expected column = \"name\" in:\n{saved}");
}

#[test]
fn test_update_filter_config_removes_filter() {
    let fs = MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "name"
condition = { contains = "fire" }
"#,
    );

    let result = update_filter_config_with_fs(&fs, "/test.toml", None);
    assert!(result.is_ok());

    let saved = fs.last_written_content().unwrap();
    assert!(!saved.contains("[metadata.filter]"), "Expected no filter section in:\n{saved}");
    assert!(saved.contains("[metadata]"), "Expected metadata section preserved in:\n{saved}");
}

#[test]
fn test_update_filter_config_preserves_other_metadata() {
    let fs = MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.sort]
column = "name"
"#,
    );
    let filter_config = FilterConfig {
        filters: vec![ColumnFilter::new("cost", FilterCondition::Boolean(true))],
        active: true,
    };

    let result = update_filter_config_with_fs(&fs, "/test.toml", Some(&filter_config));
    assert!(result.is_ok());

    let saved = fs.last_written_content().unwrap();
    assert!(saved.contains("column = \"name\""), "Expected sort config preserved in:\n{saved}");
    assert!(saved.contains("active = true"), "Expected filter config in:\n{saved}");
}

#[test]
fn test_filter_state_manager_per_file() {
    let manager = FilterStateManager::new();
    let filters_a = vec![ColumnFilterState::contains("name", "fire")];
    let filters_b = vec![ColumnFilterState::contains("cost", "3")];

    manager.set_filters("/file_a.toml", "cards", filters_a.clone());
    manager.set_filters("/file_b.toml", "cards", filters_b.clone());

    assert_eq!(manager.get_filters("/file_a.toml", "cards"), filters_a);
    assert_eq!(manager.get_filters("/file_b.toml", "cards"), filters_b);
    assert!(manager.get_filters("/file_c.toml", "cards").is_empty());
}

#[test]
fn test_filter_state_manager_clear() {
    let manager = FilterStateManager::new();
    manager.set_filters("/test.toml", "cards", vec![ColumnFilterState::contains("name", "fire")]);
    manager.set_hidden_rows("/test.toml", "cards", vec![1, 3, 5]);

    manager.clear_filters("/test.toml", "cards");
    assert!(manager.get_filters("/test.toml", "cards").is_empty());
    assert!(manager.get_hidden_rows("/test.toml", "cards").is_empty());
}

#[test]
fn test_hidden_rows_stored_and_retrieved() {
    let manager = FilterStateManager::new();
    assert!(manager.get_hidden_rows("/test.toml", "cards").is_empty());

    manager.set_hidden_rows("/test.toml", "cards", vec![0, 2, 4]);
    assert_eq!(manager.get_hidden_rows("/test.toml", "cards"), vec![0, 2, 4]);
}

#[test]
fn test_is_row_visible_with_hidden_rows() {
    let manager = FilterStateManager::new();
    manager.set_hidden_rows("/test.toml", "cards", vec![1, 3]);

    assert!(manager.is_row_visible("/test.toml", "cards", 0));
    assert!(!manager.is_row_visible("/test.toml", "cards", 1));
    assert!(manager.is_row_visible("/test.toml", "cards", 2));
    assert!(!manager.is_row_visible("/test.toml", "cards", 3));
    assert!(manager.is_row_visible("/test.toml", "cards", 4));
}

#[test]
fn test_is_row_visible_without_filters() {
    let manager = FilterStateManager::new();
    assert!(manager.is_row_visible("/test.toml", "cards", 0));
    assert!(manager.is_row_visible("/test.toml", "cards", 100));
}

#[test]
fn test_compute_hidden_rows_no_filters() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let hidden = compute_hidden_rows(&data, &[]);
    assert!(hidden.is_empty());
}

#[test]
fn test_compute_hidden_rows_contains_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "type".to_string()],
        rows: vec![
            vec![serde_json::json!("Fire Dragon"), serde_json::json!("fire")],
            vec![serde_json::json!("Water Sprite"), serde_json::json!("water")],
            vec![serde_json::json!("Fire Phoenix"), serde_json::json!("fire")],
        ],
    };

    let filters = vec![ColumnFilterState::contains("name", "Fire")];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![1]);
}

#[test]
fn test_compute_hidden_rows_equals_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "rarity".to_string()],
        rows: vec![
            vec![serde_json::json!("Card A"), serde_json::json!("common")],
            vec![serde_json::json!("Card B"), serde_json::json!("rare")],
            vec![serde_json::json!("Card C"), serde_json::json!("common")],
        ],
    };

    let filters = vec![ColumnFilterState::equals("rarity", serde_json::json!("rare"))];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![0, 2]);
}

#[test]
fn test_compute_hidden_rows_multiple_filters_and_logic() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "type".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Fire Dragon"), serde_json::json!("fire"), serde_json::json!(5)],
            vec![
                serde_json::json!("Water Sprite"),
                serde_json::json!("water"),
                serde_json::json!(3),
            ],
            vec![serde_json::json!("Fire Imp"), serde_json::json!("fire"), serde_json::json!(2)],
        ],
    };

    let filters = vec![ColumnFilterState::contains("type", "fire"), ColumnFilterState {
        column: "cost".to_string(),
        condition: FilterConditionState::Range { min: Some(3.0), max: None },
    }];

    let hidden = compute_hidden_rows(&data, &filters);
    // Water Sprite: fails type filter. Fire Imp: passes type but cost=2 < min=3.
    assert_eq!(hidden, vec![1, 2]);
}

#[test]
fn test_compute_visible_rows() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")], vec![
            serde_json::json!("Charlie"),
        ]],
    };

    let filters = vec![ColumnFilterState::contains("name", "li")];
    let visible = compute_visible_rows(&data, &filters);
    // "Alice" and "Charlie" contain "li"
    assert_eq!(visible, vec![0, 2]);
}

#[test]
fn test_compute_hidden_rows_values_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "rarity".to_string()],
        rows: vec![
            vec![serde_json::json!("Card A"), serde_json::json!("common")],
            vec![serde_json::json!("Card B"), serde_json::json!("rare")],
            vec![serde_json::json!("Card C"), serde_json::json!("legendary")],
            vec![serde_json::json!("Card D"), serde_json::json!("common")],
        ],
    };

    let filters = vec![ColumnFilterState::values("rarity", vec![
        serde_json::json!("common"),
        serde_json::json!("rare"),
    ])];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![2]);
}

#[test]
fn test_compute_hidden_rows_boolean_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "active".to_string()],
        rows: vec![
            vec![serde_json::json!("Card A"), serde_json::json!(true)],
            vec![serde_json::json!("Card B"), serde_json::json!(false)],
            vec![serde_json::json!("Card C"), serde_json::json!(true)],
        ],
    };

    let filters = vec![ColumnFilterState {
        column: "active".to_string(),
        condition: FilterConditionState::Boolean(true),
    }];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![1]);
}

#[test]
fn test_compute_hidden_rows_nonexistent_column() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!("Bob")]],
    };

    let filters = vec![ColumnFilterState::contains("nonexistent", "test")];
    let hidden = compute_hidden_rows(&data, &filters);
    assert!(hidden.is_empty());
}

#[test]
fn test_compute_hidden_rows_null_values() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![vec![serde_json::json!("Alice")], vec![serde_json::json!(null)], vec![
            serde_json::json!("Bob"),
        ]],
    };

    let filters = vec![ColumnFilterState::contains("name", "li")];
    let hidden = compute_hidden_rows(&data, &filters);
    // null doesn't contain "li", Bob doesn't contain "li"
    assert_eq!(hidden, vec![1, 2]);
}

#[test]
fn test_hidden_rows_per_file_isolation() {
    let manager = FilterStateManager::new();
    manager.set_hidden_rows("/file_a.toml", "cards", vec![0, 1]);
    manager.set_hidden_rows("/file_b.toml", "cards", vec![2, 3]);

    assert_eq!(manager.get_hidden_rows("/file_a.toml", "cards"), vec![0, 1]);
    assert_eq!(manager.get_hidden_rows("/file_b.toml", "cards"), vec![2, 3]);
    assert!(manager.get_hidden_rows("/file_c.toml", "cards").is_empty());
}

#[test]
fn test_compute_hidden_rows_range_filter() {
    let data = TomlTableData {
        headers: vec!["name".to_string(), "cost".to_string()],
        rows: vec![
            vec![serde_json::json!("Card A"), serde_json::json!(1)],
            vec![serde_json::json!("Card B"), serde_json::json!(5)],
            vec![serde_json::json!("Card C"), serde_json::json!(10)],
            vec![serde_json::json!("Card D"), serde_json::json!(3)],
        ],
    };

    let filters = vec![ColumnFilterState {
        column: "cost".to_string(),
        condition: FilterConditionState::Range { min: Some(2.0), max: Some(6.0) },
    }];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![0, 2]);
}

#[test]
fn test_compute_hidden_rows_contains_case_insensitive() {
    let data = TomlTableData {
        headers: vec!["name".to_string()],
        rows: vec![
            vec![serde_json::json!("FIRE Dragon")],
            vec![serde_json::json!("water sprite")],
            vec![serde_json::json!("Fire Phoenix")],
        ],
    };

    let filters = vec![ColumnFilterState::contains("name", "fire")];
    let hidden = compute_hidden_rows(&data, &filters);
    assert_eq!(hidden, vec![1]);
}
