use serde_json::json;

use crate::test_utils::harness::TvTestHarness;

#[test]
fn test_enum_rules_extracted_for_dropdown() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_enum.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
rarity = "Common"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event", "Artifact"]

[[metadata.validation_rules]]
column = "rarity"
type = "enum"
enum = ["Common", "Uncommon", "Rare", "Legendary"]
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 2);

    assert_eq!(enum_rules[0].column, "card_type");
    assert_eq!(enum_rules[0].allowed_values, vec!["Character", "Event", "Artifact"]);

    assert_eq!(enum_rules[1].column, "rarity");
    assert_eq!(enum_rules[1].allowed_values, vec!["Common", "Uncommon", "Rare", "Legendary"]);
}

#[test]
fn test_enum_rules_filter_non_enum() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_mixed.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "required"

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10

[[metadata.validation_rules]]
column = "cost"
type = "integer"
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 1, "Should only extract enum rules, not required/range/type");
    assert_eq!(enum_rules[0].column, "card_type");
    assert_eq!(enum_rules[0].allowed_values, vec!["Character", "Event"]);
}

#[test]
fn test_enum_rules_empty_when_no_metadata() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_no_metadata.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert!(enum_rules.is_empty());
}

#[test]
fn test_enum_rules_empty_when_no_enum_rules() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_no_enums.toml",
        r#"[[cards]]
id = "card-1"
name = "Test"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "name"
type = "required"

[[metadata.validation_rules]]
column = "name"
type = "pattern"
pattern = "^[A-Z]"
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert!(enum_rules.is_empty());
}

#[test]
fn test_enum_dropdown_rejects_invalid_value_on_save() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_reject.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event", "Artifact"]
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("NotInList"));
    assert!(result.is_err(), "Save should be rejected for value not in enum list");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(
        table.rows[0][idx].as_str(),
        Some("Character"),
        "Original value should be preserved"
    );
}

#[test]
fn test_enum_dropdown_accepts_valid_value_on_save() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_accept.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event", "Artifact"]
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Event"));
    assert!(result.is_ok(), "Save should succeed for value in enum list");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx = table.headers.iter().position(|h| h == "card_type").unwrap();
    assert_eq!(table.rows[0][idx].as_str(), Some("Event"));
}

#[test]
fn test_enum_dropdown_allows_null_value() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_null.toml",
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
    assert!(result.is_ok(), "Null should be allowed for enum columns (blank cell)");
}

#[test]
fn test_enum_dropdown_with_single_allowed_value() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_single.toml",
        r#"[[items]]
status = "Active"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "status"
type = "enum"
enum = ["Active"]
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 1);
    assert_eq!(enum_rules[0].allowed_values, vec!["Active"]);

    let result = harness.save_cell(&path, "items", 0, "status", json!("Inactive"));
    assert!(result.is_err(), "Value not in single-item list should be rejected");

    let result = harness.save_cell(&path, "items", 0, "status", json!("Active"));
    assert!(result.is_ok(), "The single allowed value should be accepted");
}

#[test]
fn test_enum_dropdown_with_many_allowed_values() {
    let harness = TvTestHarness::new();
    let values: Vec<String> = (1..=50).map(|i| format!("Option{i}")).collect();
    let values_toml: Vec<String> = values.iter().map(|v| format!("\"{v}\"")).collect();

    let content = format!(
        r#"[[items]]
choice = "Option1"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "choice"
type = "enum"
enum = [{}]
"#,
        values_toml.join(", ")
    );

    let path = harness.create_toml_file("dropdown_many.toml", &content);

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 1);
    assert_eq!(enum_rules[0].allowed_values.len(), 50);
    assert_eq!(enum_rules[0].allowed_values[0], "Option1");
    assert_eq!(enum_rules[0].allowed_values[49], "Option50");
}

#[test]
fn test_enum_dropdown_multiple_columns() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_multi_col.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"
element = "Fire"
rarity = "Common"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.validation_rules]]
column = "element"
type = "enum"
enum = ["Fire", "Water", "Earth", "Air"]

[[metadata.validation_rules]]
column = "rarity"
type = "enum"
enum = ["Common", "Uncommon", "Rare"]
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 3);

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Event"));
    assert!(result.is_ok());

    let result = harness.save_cell(&path, "cards", 0, "element", json!("Water"));
    assert!(result.is_ok());

    let result = harness.save_cell(&path, "cards", 0, "rarity", json!("Uncommon"));
    assert!(result.is_ok());

    let result = harness.save_cell(&path, "cards", 0, "element", json!("Lightning"));
    assert!(result.is_err(), "Invalid element should be rejected");
}

#[test]
fn test_enum_dropdown_custom_error_message() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_custom_msg.toml",
        r#"[[cards]]
id = "card-1"
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
message = "Please select a valid card type from the dropdown"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "card_type", json!("Invalid"));
    match result {
        Err(tv_lib::error::error_types::TvError::ValidationFailed { message, .. }) => {
            assert_eq!(message, "Please select a valid card type from the dropdown");
        }
        other => panic!("Expected ValidationFailed error, got: {other:?}"),
    }
}

#[test]
fn test_enum_dropdown_empty_allowed_values_list() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "dropdown_empty_list.toml",
        r#"[[items]]
status = "Active"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "status"
type = "enum"
enum = []
"#,
    );

    let enum_rules = harness.parse_enum_validation_rules(&path).unwrap();
    assert_eq!(enum_rules.len(), 1);
    assert!(enum_rules[0].allowed_values.is_empty());

    let result = harness.save_cell(&path, "items", 0, "status", json!("Active"));
    assert!(result.is_err(), "Any value should be rejected when allowed list is empty");
}
