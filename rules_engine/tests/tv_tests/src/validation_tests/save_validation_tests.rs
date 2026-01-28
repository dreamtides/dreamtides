use serde_json::json;
use tv_lib::error::error_types::TvError;
use tv_lib::toml::document_writer::CellUpdate;

use crate::test_utils::harness::TvTestHarness;

#[test]
fn test_save_cell_with_enum_validation_pass() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "enum_pass.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Event"));
    assert!(result.is_ok(), "Save should succeed: {:?}", result);

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(table.rows[0][idx].as_str(), Some("Event"));
}

#[test]
fn test_save_cell_with_enum_validation_fail() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "enum_fail.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Invalid"));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "card_type");
            assert_eq!(row, 0);
            assert!(message.contains("Invalid"), "Message should mention the invalid value");
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("Character"),
        "Original value should be preserved"
    );
}

#[test]
fn test_save_cell_with_range_validation_pass() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "range_pass.toml",
        r#"[[cards]]
id = "card-1"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(5));
    assert!(result.is_ok(), "Save should succeed: {:?}", result);
}

#[test]
fn test_save_cell_with_range_validation_fail() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "range_fail.toml",
        r#"[[cards]]
id = "card-1"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(15));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "cost");
            assert_eq!(row, 0);
            assert!(message.contains("greater than maximum"), "Message: {}", message);
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_cell_with_required_validation_fail() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "required_fail.toml",
        r#"[[cards]]
id = "card-1"
name = "Test Card"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "name"
type = "required"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "name", json!(""));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "name");
            assert_eq!(row, 0);
            assert!(message.contains("required"), "Message: {}", message);
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_cell_with_pattern_validation_pass() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "pattern_pass.toml",
        r#"[[cards]]
id = "AB-123"
name = "Test"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "pattern"
pattern = "^[A-Z]{2}-\\d{3}$"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "id", json!("CD-456"));
    assert!(result.is_ok(), "Save should succeed: {:?}", result);
}

#[test]
fn test_save_cell_with_pattern_validation_fail() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "pattern_fail.toml",
        r#"[[cards]]
id = "AB-123"
name = "Test"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "pattern"
pattern = "^[A-Z]{2}-\\d{3}$"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "id", json!("invalid"));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "id");
            assert_eq!(row, 0);
            assert!(message.contains("does not match pattern"), "Message: {}", message);
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_cell_with_type_validation_fail() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "type_fail.toml",
        r#"[[cards]]
id = "card-1"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "integer"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "cost", json!("not a number"));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "cost");
            assert_eq!(row, 0);
            assert!(message.contains("not a valid integer"), "Message: {}", message);
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_cell_no_validation_rules() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "no_rules.toml",
        r#"[[cards]]
id = "card-1"
name = "Test"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "name", json!("Any value"));
    assert!(result.is_ok(), "Save should succeed without validation rules");
}

#[test]
fn test_save_cell_validation_with_custom_message() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "custom_message.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
message = "Please select a valid card type"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Invalid"));
    match result {
        Err(TvError::ValidationFailed { message, .. }) => {
            assert_eq!(message, "Please select a valid card type");
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_cell_multiple_rules_all_pass() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "multi_rules_pass.toml",
        r#"[[cards]]
id = "card-1"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "required"

[[metadata.validation_rules]]
column = "cost"
type = "integer"

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(5));
    assert!(result.is_ok(), "Save should succeed when all rules pass");
}

#[test]
fn test_save_cell_multiple_rules_one_fails() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "multi_rules_fail.toml",
        r#"[[cards]]
id = "card-1"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "required"

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(15));
    match result {
        Err(TvError::ValidationFailed { .. }) => {}
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_save_batch_with_validation_all_pass() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch_pass.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
cost = 3

[[cards]]
id = "card-2"
card_type = "Event"
cost = 5

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "card_type".to_string(), value: json!("Event") },
        CellUpdate { row_index: 1, column_key: "cost".to_string(), value: json!(7) },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok(), "Batch save should succeed: {:?}", result);
    let batch_result = result.unwrap();
    assert!(batch_result.success);
    assert_eq!(batch_result.applied_count, 2);
}

#[test]
fn test_save_batch_with_validation_one_fails() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch_one_fail.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
cost = 3

[[cards]]
id = "card-2"
card_type = "Event"
cost = 5

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "card_type".to_string(), value: json!("Invalid") },
        CellUpdate { row_index: 1, column_key: "cost".to_string(), value: json!(7) },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok());
    let batch_result = result.unwrap();
    assert!(!batch_result.success, "Batch should fail when validation fails");
    assert_eq!(batch_result.applied_count, 0);
    assert_eq!(batch_result.failed_count, 1);
    assert_eq!(batch_result.failed_updates[0].column_key, "card_type");
    assert!(batch_result.failed_updates[0].reason.contains("Invalid"));

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("Character"),
        "Original value should be preserved"
    );
}

#[test]
fn test_save_batch_with_validation_multiple_fails() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch_multi_fail.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "card_type".to_string(), value: json!("Invalid") },
        CellUpdate { row_index: 0, column_key: "cost".to_string(), value: json!(100) },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok());
    let batch_result = result.unwrap();
    assert!(!batch_result.success);
    assert_eq!(batch_result.failed_count, 2);
}

#[test]
fn test_save_cell_null_bypasses_non_required_validation() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "null_bypass.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", serde_json::Value::Null);
    assert!(result.is_ok(), "Null should bypass enum validation: {:?}", result);
}

#[test]
fn test_save_cell_null_fails_required_validation() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "null_required.toml",
        r#"[[cards]]
id = "card-1"
name = "Test"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "name"
type = "required"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "name", serde_json::Value::Null);
    match result {
        Err(TvError::ValidationFailed { column, .. }) => {
            assert_eq!(column, "name");
        }
        other => panic!("Expected ValidationFailed error, got: {:?}", other),
    }
}
