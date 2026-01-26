use serde_json::json;
use tv_lib::validation::validation_rules::{ValidationRule, ValueType};
use tv_lib::validation::validators::{first_error, is_valid, validate, validate_all};

#[test]
fn test_enum_validation_pass() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("Character"));
    assert!(result.valid);
    assert!(result.error_message.is_none());
}

#[test]
fn test_enum_validation_fail() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("Invalid"));
    assert!(!result.valid);
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("Invalid"));
}

#[test]
fn test_enum_validation_null_passes() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!(null));
    assert!(result.valid);
}

#[test]
fn test_enum_validation_custom_message() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string()],
        message: Some("Please select a valid card type".to_string()),
    };

    let result = validate(&rule, &json!("Invalid"));
    assert!(!result.valid);
    assert_eq!(result.error_message.as_deref(), Some("Please select a valid card type"));
}

#[test]
fn test_range_validation_pass() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(5));
    assert!(result.valid);
}

#[test]
fn test_range_validation_fail_below_min() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(-5));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("less than minimum"));
}

#[test]
fn test_range_validation_fail_above_max() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(15));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("greater than maximum"));
}

#[test]
fn test_range_validation_min_only() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: None,
        message: None,
    };

    let result = validate(&rule, &json!(1000));
    assert!(result.valid);
}

#[test]
fn test_range_validation_max_only() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: None,
        max: Some(100.0),
        message: None,
    };

    let result = validate(&rule, &json!(-50));
    assert!(result.valid);
}

#[test]
fn test_range_validation_null_passes() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!(null));
    assert!(result.valid);
}

#[test]
fn test_range_validation_float() {
    let rule = ValidationRule::Range {
        column: "multiplier".to_string(),
        min: Some(0.0),
        max: Some(1.0),
        message: None,
    };

    let result = validate(&rule, &json!(0.5));
    assert!(result.valid);
}

#[test]
fn test_range_validation_string_number() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!("5"));
    assert!(result.valid);
}

#[test]
fn test_range_validation_non_numeric_string() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };

    let result = validate(&rule, &json!("not a number"));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("not a valid number"));
}

#[test]
fn test_pattern_validation_pass() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!("AB-123"));
    assert!(result.valid);
}

#[test]
fn test_pattern_validation_fail() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!("invalid"));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("does not match pattern"));
}

#[test]
fn test_pattern_validation_null_passes() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!(null));
    assert!(result.valid);
}

#[test]
fn test_pattern_validation_invalid_regex() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"[invalid".to_string(),
        message: None,
    };

    let result = validate(&rule, &json!("test"));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("Invalid regex pattern"));
}

#[test]
fn test_pattern_validation_custom_message() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: Some("ID must be in format XX-000".to_string()),
    };

    let result = validate(&rule, &json!("invalid"));
    assert!(!result.valid);
    assert_eq!(result.error_message.as_deref(), Some("ID must be in format XX-000"));
}

#[test]
fn test_required_validation_pass() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!("Test Name"));
    assert!(result.valid);
}

#[test]
fn test_required_validation_fail_null() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!(null));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("required"));
}

#[test]
fn test_required_validation_fail_empty_string() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!(""));
    assert!(!result.valid);
}

#[test]
fn test_required_validation_fail_whitespace_only() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };

    let result = validate(&rule, &json!("   "));
    assert!(!result.valid);
}

#[test]
fn test_required_validation_pass_number() {
    let rule = ValidationRule::Required { column: "cost".to_string(), message: None };

    let result = validate(&rule, &json!(0));
    assert!(result.valid);
}

#[test]
fn test_required_validation_pass_boolean() {
    let rule = ValidationRule::Required { column: "active".to_string(), message: None };

    let result = validate(&rule, &json!(false));
    assert!(result.valid);
}

#[test]
fn test_required_validation_custom_message() {
    let rule = ValidationRule::Required {
        column: "name".to_string(),
        message: Some("Name cannot be empty".to_string()),
    };

    let result = validate(&rule, &json!(null));
    assert!(!result.valid);
    assert_eq!(result.error_message.as_deref(), Some("Name cannot be empty"));
}

#[test]
fn test_type_validation_string_pass() {
    let rule = ValidationRule::Type {
        column: "name".to_string(),
        value_type: ValueType::String,
        message: None,
    };

    let result = validate(&rule, &json!("Test"));
    assert!(result.valid);
}

#[test]
fn test_type_validation_string_fail() {
    let rule = ValidationRule::Type {
        column: "name".to_string(),
        value_type: ValueType::String,
        message: None,
    };

    let result = validate(&rule, &json!(42));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("not a valid string"));
}

#[test]
fn test_type_validation_integer_pass() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(42));
    assert!(result.valid);
}

#[test]
fn test_type_validation_integer_from_float_with_zero_fraction() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(42.0));
    assert!(result.valid);
}

#[test]
fn test_type_validation_integer_fail_float() {
    let rule = ValidationRule::Type {
        column: "cost".to_string(),
        value_type: ValueType::Integer,
        message: None,
    };

    let result = validate(&rule, &json!(3.14));
    assert!(!result.valid);
}

#[test]
fn test_type_validation_float_pass() {
    let rule = ValidationRule::Type {
        column: "multiplier".to_string(),
        value_type: ValueType::Float,
        message: None,
    };

    let result = validate(&rule, &json!(3.14));
    assert!(result.valid);
}

#[test]
fn test_type_validation_float_accepts_integer() {
    let rule = ValidationRule::Type {
        column: "multiplier".to_string(),
        value_type: ValueType::Float,
        message: None,
    };

    let result = validate(&rule, &json!(42));
    assert!(result.valid);
}

#[test]
fn test_type_validation_boolean_pass() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!(true));
    assert!(result.valid);
}

#[test]
fn test_type_validation_boolean_fail() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };

    let result = validate(&rule, &json!("true"));
    assert!(!result.valid);
    assert!(result.error_message.unwrap().contains("not a valid boolean"));
}

#[test]
fn test_type_validation_null_passes() {
    let rule = ValidationRule::Type {
        column: "name".to_string(),
        value_type: ValueType::String,
        message: None,
    };

    let result = validate(&rule, &json!(null));
    assert!(result.valid);
}

#[test]
fn test_validate_all_filters_by_column() {
    let rules = vec![
        ValidationRule::Required { column: "name".to_string(), message: None },
        ValidationRule::Required { column: "cost".to_string(), message: None },
        ValidationRule::Range {
            column: "cost".to_string(),
            min: Some(0.0),
            max: Some(10.0),
            message: None,
        },
    ];

    let results = validate_all(&rules, "cost", &json!(5));
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.valid));
}

#[test]
fn test_validate_all_multiple_rules_all_pass() {
    let rules = vec![
        ValidationRule::Required { column: "name".to_string(), message: None },
        ValidationRule::Type {
            column: "name".to_string(),
            value_type: ValueType::String,
            message: None,
        },
        ValidationRule::Pattern {
            column: "name".to_string(),
            pattern: r"^[A-Z].*".to_string(),
            message: None,
        },
    ];

    let results = validate_all(&rules, "name", &json!("Test"));
    assert_eq!(results.len(), 3);
    assert!(is_valid(&results));
}

#[test]
fn test_validate_all_multiple_rules_one_fails() {
    let rules = vec![
        ValidationRule::Required { column: "name".to_string(), message: None },
        ValidationRule::Pattern {
            column: "name".to_string(),
            pattern: r"^[A-Z].*".to_string(),
            message: None,
        },
    ];

    let results = validate_all(&rules, "name", &json!("lowercase"));
    assert_eq!(results.len(), 2);
    assert!(!is_valid(&results));
    assert!(first_error(&results).is_some());
}

#[test]
fn test_is_valid_all_pass() {
    let rules = vec![ValidationRule::Required { column: "name".to_string(), message: None }];

    let results = validate_all(&rules, "name", &json!("Test"));
    assert!(is_valid(&results));
}

#[test]
fn test_is_valid_one_fails() {
    let rules = vec![ValidationRule::Required { column: "name".to_string(), message: None }];

    let results = validate_all(&rules, "name", &json!(null));
    assert!(!is_valid(&results));
}

#[test]
fn test_first_error_returns_none_when_all_valid() {
    let rules = vec![ValidationRule::Required { column: "name".to_string(), message: None }];

    let results = validate_all(&rules, "name", &json!("Test"));
    assert!(first_error(&results).is_none());
}

#[test]
fn test_first_error_returns_first_failure() {
    let rules = vec![
        ValidationRule::Required { column: "name".to_string(), message: None },
        ValidationRule::Pattern {
            column: "name".to_string(),
            pattern: r"^[A-Z].*".to_string(),
            message: Some("Must start with uppercase".to_string()),
        },
    ];

    let results = validate_all(&rules, "name", &json!(""));
    let error = first_error(&results);
    assert!(error.is_some());
    assert_eq!(error.unwrap().rule_type, "required");
}

#[test]
fn test_validation_result_has_correct_metadata() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string()],
        message: None,
    };

    let result = validate(&rule, &json!("Invalid"));
    assert_eq!(result.column, "card_type");
    assert_eq!(result.rule_type, "enum");
}

#[test]
fn test_rule_type_name_enum() {
    let rule =
        ValidationRule::Enum { column: "col".to_string(), allowed_values: vec![], message: None };
    assert_eq!(rule.rule_type_name(), "enum");
}

#[test]
fn test_rule_type_name_range() {
    let rule =
        ValidationRule::Range { column: "col".to_string(), min: None, max: None, message: None };
    assert_eq!(rule.rule_type_name(), "range");
}

#[test]
fn test_rule_type_name_pattern() {
    let rule = ValidationRule::Pattern {
        column: "col".to_string(),
        pattern: ".*".to_string(),
        message: None,
    };
    assert_eq!(rule.rule_type_name(), "pattern");
}

#[test]
fn test_rule_type_name_required() {
    let rule = ValidationRule::Required { column: "col".to_string(), message: None };
    assert_eq!(rule.rule_type_name(), "required");
}

#[test]
fn test_rule_type_name_type() {
    let rule = ValidationRule::Type {
        column: "col".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };
    assert_eq!(rule.rule_type_name(), "type");
}

#[test]
fn test_describe_enum() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };
    assert_eq!(rule.describe(), "Must be one of: Character, Event");
}

#[test]
fn test_describe_range_both_bounds() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };
    assert_eq!(rule.describe(), "Must be between 0 and 10");
}

#[test]
fn test_describe_range_min_only() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: None,
        message: None,
    };
    assert_eq!(rule.describe(), "Must be at least 0");
}

#[test]
fn test_describe_range_max_only() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: None,
        max: Some(100.0),
        message: None,
    };
    assert_eq!(rule.describe(), "Must be at most 100");
}

#[test]
fn test_describe_pattern() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^[A-Z]{2}-\d{3}$".to_string(),
        message: None,
    };
    assert!(rule.describe().contains(r"^[A-Z]{2}-\d{3}$"));
}

#[test]
fn test_describe_required() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };
    assert_eq!(rule.describe(), "This field is required");
}

#[test]
fn test_describe_type() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };
    assert_eq!(rule.describe(), "Must be a boolean value");
}

#[test]
fn test_display_value_type() {
    assert_eq!(format!("{}", ValueType::String), "string");
    assert_eq!(format!("{}", ValueType::Integer), "integer");
    assert_eq!(format!("{}", ValueType::Float), "float");
    assert_eq!(format!("{}", ValueType::Boolean), "boolean");
}

#[test]
fn test_display_validation_rule_enum() {
    let rule = ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: None,
    };
    assert_eq!(format!("{rule}"), "enum(card_type: [Character, Event])");
}

#[test]
fn test_display_validation_rule_range() {
    let rule = ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    };
    assert_eq!(format!("{rule}"), "range(cost: 0..10)");
}

#[test]
fn test_display_validation_rule_pattern() {
    let rule = ValidationRule::Pattern {
        column: "id".to_string(),
        pattern: r"^\d+$".to_string(),
        message: None,
    };
    assert_eq!(format!("{rule}"), r"pattern(id: /^\d+$/)");
}

#[test]
fn test_display_validation_rule_required() {
    let rule = ValidationRule::Required { column: "name".to_string(), message: None };
    assert_eq!(format!("{rule}"), "required(name)");
}

#[test]
fn test_display_validation_rule_type() {
    let rule = ValidationRule::Type {
        column: "active".to_string(),
        value_type: ValueType::Boolean,
        message: None,
    };
    assert_eq!(format!("{rule}"), "type(active: boolean)");
}
