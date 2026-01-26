use serde_json::json;
use tv_lib::error::error_types::TvError;
use tv_lib::validation::validation_rules::{ValidationRule, ValueType};
use tv_lib::validation::validators::validate;

use crate::test_utils::test_utils_mod::TvTestHarness;

#[test]
fn test_type_validation_integer_pass() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(42));
    assert!(result.valid, "Integer value 42 should pass integer type validation");
}

#[test]
fn test_type_validation_integer_string_fail() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!("abc"));
    assert!(!result.valid, "String 'abc' should fail integer type validation");
    assert!(
        result.error_message.as_deref().unwrap().contains("not a valid integer"),
        "Error message should mention invalid integer"
    );
}

#[test]
fn test_type_validation_integer_string_number_fail() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!("42"));
    assert!(
        !result.valid,
        "String '42' should fail integer type validation because it is a string, not a number"
    );
}

#[test]
fn test_type_validation_integer_float_fraction_fail() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(3.14));
    assert!(!result.valid, "Float 3.14 should fail integer type validation");
}

#[test]
fn test_type_validation_integer_float_zero_fraction_pass() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(42.0));
    assert!(result.valid, "Float 42.0 with zero fraction should pass integer validation");
}

#[test]
fn test_type_validation_integer_negative_pass() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(-5));
    assert!(result.valid, "Negative integer -5 should pass integer type validation");
}

#[test]
fn test_type_validation_integer_zero_pass() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(0));
    assert!(result.valid, "Zero should pass integer type validation");
}

#[test]
fn test_type_validation_boolean_true_pass() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!(true));
    assert!(result.valid, "Boolean true should pass boolean type validation");
}

#[test]
fn test_type_validation_boolean_false_pass() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!(false));
    assert!(result.valid, "Boolean false should pass boolean type validation");
}

#[test]
fn test_type_validation_boolean_string_true_fail() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!("true"));
    assert!(!result.valid, "String 'true' should fail boolean type validation");
    assert!(
        result.error_message.as_deref().unwrap().contains("not a valid boolean"),
        "Error message should mention invalid boolean"
    );
}

#[test]
fn test_type_validation_boolean_string_false_fail() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!("false"));
    assert!(!result.valid, "String 'false' should fail boolean type validation");
}

#[test]
fn test_type_validation_boolean_integer_fail() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!(1));
    assert!(!result.valid, "Integer 1 should fail boolean type validation");
}

#[test]
fn test_enum_validation_allowed_value_pass() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string(), "Artifact".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("Character"));
    assert!(result.valid, "Value in allowed list should pass");

    let result = validate(&rule, &json!("Event"));
    assert!(result.valid, "Value in allowed list should pass");

    let result = validate(&rule, &json!("Artifact"));
    assert!(result.valid, "Value in allowed list should pass");
}

#[test]
fn test_enum_validation_disallowed_value_fail() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("Spell"));
    assert!(!result.valid, "Value not in allowed list should fail");
    assert!(
        result.error_message.as_deref().unwrap().contains("Spell"),
        "Error message should mention the invalid value"
    );
}

#[test]
fn test_enum_case_sensitive() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("character"));
    assert!(
        !result.valid,
        "Enum validation is case-sensitive; 'character' should fail for 'Character'"
    );

    let result = validate(&rule, &json!("CHARACTER"));
    assert!(
        !result.valid,
        "Enum validation is case-sensitive; 'CHARACTER' should fail for 'Character'"
    );

    let result = validate(&rule, &json!("event"));
    assert!(!result.valid, "Enum validation is case-sensitive; 'event' should fail for 'Event'");

    let result = validate(&rule, &json!("Character"));
    assert!(result.valid, "Exact case match 'Character' should pass");
}

#[test]
fn test_range_validation_within_bounds_pass() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(5));
    assert!(result.valid, "Value within bounds should pass");
}

#[test]
fn test_range_validation_at_min_boundary_pass() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(0));
    assert!(result.valid, "Value at minimum boundary should pass");
}

#[test]
fn test_range_validation_at_max_boundary_pass() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(10));
    assert!(result.valid, "Value at maximum boundary should pass");
}

#[test]
fn test_range_validation_below_min_fail() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(-1));
    assert!(!result.valid, "Value below minimum should fail");
    assert!(
        result.error_message.as_deref().unwrap().contains("less than minimum"),
        "Error message should mention minimum violation"
    );
}

#[test]
fn test_range_validation_above_max_fail() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(11));
    assert!(!result.valid, "Value above maximum should fail");
    assert!(
        result.error_message.as_deref().unwrap().contains("greater than maximum"),
        "Error message should mention maximum violation"
    );
}

#[test]
fn test_range_validation_non_numeric_string_fail() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!("abc"));
    assert!(!result.valid, "Non-numeric string should fail range validation");
    assert!(
        result.error_message.as_deref().unwrap().contains("not a valid number"),
        "Error message should mention invalid number"
    );
}

#[test]
fn test_pattern_validation_matching_pass() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!("AB-123"));
    assert!(result.valid, "Value matching pattern should pass");
}

#[test]
fn test_pattern_validation_not_matching_fail() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!("invalid-id"));
    assert!(!result.valid, "Value not matching pattern should fail");
    assert!(
        result.error_message.as_deref().unwrap().contains("does not match pattern"),
        "Error message should mention pattern mismatch"
    );
}

#[test]
fn test_required_validation_non_empty_pass() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!("Test Name"));
    assert!(result.valid, "Non-empty string should pass required validation");
}

#[test]
fn test_required_validation_empty_string_fail() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!(""));
    assert!(!result.valid, "Empty string should fail required validation");
    assert!(
        result.error_message.as_deref().unwrap().contains("required"),
        "Error message should mention required"
    );
}

#[test]
fn test_required_validation_null_fail() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!(null));
    assert!(!result.valid, "Null should fail required validation");
}

#[test]
fn test_required_validation_whitespace_only_fail() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!("   "));
    assert!(!result.valid, "Whitespace-only string should fail required validation");
}

#[test]
fn test_save_rejects_invalid_integer_type() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_integer_save.toml",
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

    let result = harness.save_cell(&path, "cards", 0, "cost", json!("abc"));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "cost");
            assert_eq!(row, 0);
            assert!(message.contains("not a valid integer"), "Message: {message}");
        }
        other => panic!("Expected ValidationFailed error, got: {other:?}"),
    }

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "cost").unwrap();
    assert_eq!(
        table.rows[0][idx],
        json!(3),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_rejects_invalid_boolean_type() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_boolean_save.toml",
        r#"[[cards]]
id = "card-1"
active = true

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "active"
type = "boolean"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "active", json!("true"));
    match result {
        Err(TvError::ValidationFailed { column, row, message }) => {
            assert_eq!(column, "active");
            assert_eq!(row, 0);
            assert!(message.contains("not a valid boolean"), "Message: {message}");
        }
        other => panic!("Expected ValidationFailed error, got: {other:?}"),
    }

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "active").unwrap();
    assert_eq!(
        table.rows[0][idx],
        json!(true),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_rejects_out_of_range_value() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_range_save.toml",
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

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(99));
    assert!(result.is_err(), "Out-of-range value should be rejected");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "cost").unwrap();
    assert_eq!(
        table.rows[0][idx],
        json!(3),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_rejects_empty_required_field() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_required_save.toml",
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
    assert!(result.is_err(), "Empty value in required field should be rejected");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "name").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("Test Card"),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_rejects_enum_wrong_case() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_enum_case_save.toml",
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

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("character"));
    assert!(
        result.is_err(),
        "Enum validation is case-sensitive; wrong-case value should be rejected"
    );

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("Character"),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_rejects_pattern_mismatch() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_pattern_save.toml",
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

    let result = harness.save_cell(&path, "cards", 0, "id", json!("bad-id"));
    assert!(result.is_err(), "Pattern-mismatching value should be rejected");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("AB-123"),
        "Original value should be preserved after rejected save"
    );
}

#[test]
fn test_save_accepts_valid_value_with_multiple_rules() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "rule_multi_pass.toml",
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

    let result = harness.save_cell(&path, "cards", 0, "cost", json!(7));
    assert!(result.is_ok(), "Value passing all rules should be accepted: {result:?}");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "cost").unwrap();
    assert_eq!(table.rows[0][idx], json!(7), "New value should be saved");
}
