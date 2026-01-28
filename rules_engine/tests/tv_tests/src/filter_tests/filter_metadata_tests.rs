use std::sync::Arc;

use tv_lib::toml::metadata::parse_filter_config_from_content;
use tv_lib::toml::metadata_serializer::update_filter_config;
use tv_lib::toml::metadata_types::{ColumnFilter, FilterCondition, FilterConfig};
use tv_lib::traits::TvConfig;

use crate::test_utils::mock_filesystem::MockFileSystem;

#[test]
fn test_parse_filter_config_with_contains_filter() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "name"

[metadata.filter.filters.condition]
contains = "fire"
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert!(config.active);
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.filters[0].column, "name");
    assert_eq!(config.filters[0].condition, FilterCondition::Contains("fire".to_string()));
}

#[test]
fn test_parse_filter_config_with_equals_filter() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "rarity"

[metadata.filter.filters.condition]
equals = "Common"
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.filters[0].condition, FilterCondition::Equals(serde_json::json!("Common")));
}

#[test]
fn test_parse_filter_config_with_range_filter() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "cost"

[metadata.filter.filters.condition]
min = 1.0
max = 10.0
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.filters[0].condition, FilterCondition::Range {
        min: Some(1.0),
        max: Some(10.0)
    });
}

#[test]
fn test_parse_filter_config_with_boolean_filter() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "active"

[metadata.filter.filters.condition]
boolean = true
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.filters[0].condition, FilterCondition::Boolean(true));
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

[[metadata.filter.filters]]
column = "name"

[metadata.filter.filters.condition]
contains = "fire"
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert!(!config.active);
    assert_eq!(config.filters.len(), 1);
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
fn test_parse_filter_config_empty_filters() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = false
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
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

[metadata.filter.filters.condition]
contains = "fire"

[[metadata.filter.filters]]
column = "active"

[metadata.filter.filters.condition]
boolean = true
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert!(config.active);
    assert_eq!(config.filters.len(), 2);
    assert_eq!(config.filters[0].column, "name");
    assert_eq!(config.filters[1].column, "active");
}

#[test]
fn test_update_filter_config_adds_filter_section() {
    let fs = MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
"#,
    );
    let config = TvConfig::new(Arc::new(fs));
    let filter_config = FilterConfig {
        filters: vec![ColumnFilter::new("name", FilterCondition::Contains("fire".to_string()))],
        active: true,
    };

    let result = update_filter_config(&config, "/test.toml", Some(&filter_config));
    assert!(result.is_ok());

    let saved = config
        .fs()
        .as_any()
        .downcast_ref::<MockFileSystem>()
        .unwrap()
        .last_written_content()
        .unwrap();
    assert!(saved.contains("[metadata]"), "Expected [metadata] section in:\n{saved}");
    assert!(saved.contains("column = \"name\""), "Expected column = \"name\" in:\n{saved}");
    assert!(saved.contains("active = true"), "Expected active = true in:\n{saved}");
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

[metadata.filter.filters.condition]
contains = "fire"
"#,
    );
    let config = TvConfig::new(Arc::new(fs));

    let result = update_filter_config(&config, "/test.toml", None);
    assert!(result.is_ok());

    let saved = config
        .fs()
        .as_any()
        .downcast_ref::<MockFileSystem>()
        .unwrap()
        .last_written_content()
        .unwrap();
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
    let config = TvConfig::new(Arc::new(fs));
    let filter_config = FilterConfig {
        filters: vec![ColumnFilter::new("active", FilterCondition::Boolean(true))],
        active: true,
    };

    let result = update_filter_config(&config, "/test.toml", Some(&filter_config));
    assert!(result.is_ok());

    let saved = config
        .fs()
        .as_any()
        .downcast_ref::<MockFileSystem>()
        .unwrap()
        .last_written_content()
        .unwrap();
    assert!(saved.contains("column = \"name\""), "Expected sort config preserved in:\n{saved}");
    assert!(saved.contains("active = true"), "Expected filter active in:\n{saved}");
}

#[test]
fn test_parse_filter_config_range_integer_values() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "cost"

[metadata.filter.filters.condition]
min = 1
max = 10
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert_eq!(config.filters[0].condition, FilterCondition::Range {
        min: Some(1.0),
        max: Some(10.0)
    });
}

#[test]
fn test_parse_filter_config_range_min_only() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[metadata.filter]
active = true

[[metadata.filter.filters]]
column = "cost"

[metadata.filter.filters.condition]
min = 5.0
"#;

    let result = parse_filter_config_from_content(content, "test.toml").unwrap();
    let config = result.expect("Expected filter config");
    assert_eq!(config.filters[0].condition, FilterCondition::Range { min: Some(5.0), max: None });
}
