use tv_lib::toml::metadata_parser::parse_validation_rules_from_content;
use tv_lib::validation::validation_rules::{ValidationRule, ValueType};

#[test]
fn test_parse_enum_rule() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event", "Artifact"]
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Enum { column, allowed_values, message } => {
            assert_eq!(column, "card_type");
            assert_eq!(allowed_values, &vec![
                "Character".to_string(),
                "Event".to_string(),
                "Artifact".to_string()
            ]);
            assert!(message.is_none());
        }
        other => panic!("Expected Enum rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_range_rule() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Range { column, min, max, message } => {
            assert_eq!(column, "cost");
            assert_eq!(*min, Some(0.0));
            assert_eq!(*max, Some(10.0));
            assert!(message.is_none());
        }
        other => panic!("Expected Range rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_range_rule_float_bounds() {
    let content = r#"
[[items]]
value = 0.5

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "value"
type = "range"
min = 0.0
max = 1.0
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Range { min, max, .. } => {
            assert!((min.unwrap() - 0.0).abs() < 0.001);
            assert!((max.unwrap() - 1.0).abs() < 0.001);
        }
        other => panic!("Expected Range rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_pattern_rule() {
    let content = r#"
[[cards]]
id = "AB-123"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "pattern"
pattern = "^[A-Z]{2}-\\d{3}$"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Pattern { column, pattern, message } => {
            assert_eq!(column, "id");
            assert_eq!(pattern, r"^[A-Z]{2}-\d{3}$");
            assert!(message.is_none());
        }
        other => panic!("Expected Pattern rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_required_rule() {
    let content = r#"
[[cards]]
name = "Test"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "name"
type = "required"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Required { column, message } => {
            assert_eq!(column, "name");
            assert!(message.is_none());
        }
        other => panic!("Expected Required rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_type_rules() {
    let content = r#"
[[items]]
a = "text"
b = 42
c = 3.14
d = true

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "a"
type = "string"

[[metadata.validation_rules]]
column = "b"
type = "integer"

[[metadata.validation_rules]]
column = "c"
type = "float"

[[metadata.validation_rules]]
column = "d"
type = "boolean"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 4);

    match &rules[0] {
        ValidationRule::Type { column, value_type, .. } => {
            assert_eq!(column, "a");
            assert_eq!(*value_type, ValueType::String);
        }
        other => panic!("Expected Type rule, got: {:?}", other),
    }

    match &rules[1] {
        ValidationRule::Type { column, value_type, .. } => {
            assert_eq!(column, "b");
            assert_eq!(*value_type, ValueType::Integer);
        }
        other => panic!("Expected Type rule, got: {:?}", other),
    }

    match &rules[2] {
        ValidationRule::Type { column, value_type, .. } => {
            assert_eq!(column, "c");
            assert_eq!(*value_type, ValueType::Float);
        }
        other => panic!("Expected Type rule, got: {:?}", other),
    }

    match &rules[3] {
        ValidationRule::Type { column, value_type, .. } => {
            assert_eq!(column, "d");
            assert_eq!(*value_type, ValueType::Boolean);
        }
        other => panic!("Expected Type rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_rule_with_custom_message() {
    let content = r#"
[[cards]]
card_type = "Character"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]
message = "Please select a valid card type"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Enum { message, .. } => {
            assert_eq!(message.as_deref(), Some("Please select a valid card type"));
        }
        other => panic!("Expected Enum rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_multiple_rules() {
    let content = r#"
[[cards]]
id = "card-1"
name = "Test"
cost = 3

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "required"

[[metadata.validation_rules]]
column = "name"
type = "required"

[[metadata.validation_rules]]
column = "cost"
type = "range"
min = 0
max = 10

[[metadata.validation_rules]]
column = "cost"
type = "integer"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 4);
}

#[test]
fn test_parse_no_metadata_section() {
    let content = r#"
[[cards]]
id = "card-1"
name = "Test"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert!(rules.is_empty());
}

#[test]
fn test_parse_no_validation_rules() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert!(rules.is_empty());
}

#[test]
fn test_parse_empty_validation_rules() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1
validation_rules = []
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert!(rules.is_empty());
}

#[test]
fn test_parse_skips_invalid_rules() {
    let content = r#"
[[cards]]
id = "card-1"

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "id"
type = "required"

[[metadata.validation_rules]]
column = "cost"
type = "unknown_type"

[[metadata.validation_rules]]
column = "name"
type = "required"
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 2, "Should skip invalid rule and parse valid ones");
}

#[test]
fn test_parse_range_min_only() {
    let content = r#"
[[items]]
value = 5

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "value"
type = "range"
min = 0
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Range { min, max, .. } => {
            assert_eq!(*min, Some(0.0));
            assert!(max.is_none());
        }
        other => panic!("Expected Range rule, got: {:?}", other),
    }
}

#[test]
fn test_parse_range_max_only() {
    let content = r#"
[[items]]
value = 5

[metadata]
schema_version = 1

[[metadata.validation_rules]]
column = "value"
type = "range"
max = 100
"#;

    let rules = parse_validation_rules_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);

    match &rules[0] {
        ValidationRule::Range { min, max, .. } => {
            assert!(min.is_none());
            assert_eq!(*max, Some(100.0));
        }
        other => panic!("Expected Range rule, got: {:?}", other),
    }
}
