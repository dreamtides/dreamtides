use tv_lib::toml::value_converter::{
    json_to_toml_edit, json_to_toml_edit_preserving_type, json_to_toml_value, toml_to_json,
};

#[test]
fn test_toml_string_to_json() {
    let toml_val = toml::Value::String("hello".to_string());
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::Value::String("hello".to_string()));
}

#[test]
fn test_toml_integer_to_json() {
    let toml_val = toml::Value::Integer(42);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::json!(42));
}

#[test]
fn test_toml_float_to_json() {
    let toml_val = toml::Value::Float(3.14);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::json!(3.14));
}

#[test]
fn test_toml_nan_to_json() {
    let toml_val = toml::Value::Float(f64::NAN);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::Value::String("NaN".to_string()));
}

#[test]
fn test_toml_infinity_to_json() {
    let toml_val = toml::Value::Float(f64::INFINITY);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::Value::String("Infinity".to_string()));
}

#[test]
fn test_toml_neg_infinity_to_json() {
    let toml_val = toml::Value::Float(f64::NEG_INFINITY);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::Value::String("-Infinity".to_string()));
}

#[test]
fn test_toml_boolean_to_json() {
    let toml_val = toml::Value::Boolean(true);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::Value::Bool(true));
}

#[test]
fn test_toml_array_to_json() {
    let toml_val = toml::Value::Array(vec![
        toml::Value::Integer(1),
        toml::Value::Integer(2),
        toml::Value::Integer(3),
    ]);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::json!([1, 2, 3]));
}

#[test]
fn test_toml_table_to_json() {
    let mut table = toml::map::Map::new();
    table.insert("key".to_string(), toml::Value::String("value".to_string()));
    let toml_val = toml::Value::Table(table);
    let json_val = toml_to_json(&toml_val);
    assert_eq!(json_val, serde_json::json!({"key": "value"}));
}

#[test]
fn test_json_null_to_toml() {
    let json_val = serde_json::Value::Null;
    assert!(json_to_toml_edit(&json_val).is_none());
}

#[test]
fn test_json_bool_to_toml() {
    let json_val = serde_json::Value::Bool(false);
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Boolean(b)) = toml_item {
        assert!(!*b.value());
    } else {
        panic!("Expected boolean value");
    }
}

#[test]
fn test_json_integer_to_toml() {
    let json_val = serde_json::json!(42);
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Integer(i)) = toml_item {
        assert_eq!(*i.value(), 42);
    } else {
        panic!("Expected integer value");
    }
}

#[test]
fn test_json_float_to_toml() {
    let json_val = serde_json::json!(3.14);
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Float(f)) = toml_item {
        assert!((f.value() - 3.14).abs() < f64::EPSILON);
    } else {
        panic!("Expected float value");
    }
}

#[test]
fn test_json_string_to_toml() {
    let json_val = serde_json::Value::String("test".to_string());
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::String(s)) = toml_item {
        assert_eq!(s.value(), "test");
    } else {
        panic!("Expected string value");
    }
}

#[test]
fn test_json_nan_string_to_toml() {
    let json_val = serde_json::Value::String("NaN".to_string());
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Float(f)) = toml_item {
        assert!(f.value().is_nan());
    } else {
        panic!("Expected float value for NaN");
    }
}

#[test]
fn test_json_infinity_string_to_toml() {
    let json_val = serde_json::Value::String("Infinity".to_string());
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Float(f)) = toml_item {
        assert!(f.value().is_infinite() && f.value().is_sign_positive());
    } else {
        panic!("Expected float value for Infinity");
    }
}

#[test]
fn test_json_neg_infinity_string_to_toml() {
    let json_val = serde_json::Value::String("-Infinity".to_string());
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Float(f)) = toml_item {
        assert!(f.value().is_infinite() && f.value().is_sign_negative());
    } else {
        panic!("Expected float value for -Infinity");
    }
}

#[test]
fn test_json_array_to_toml() {
    let json_val = serde_json::json!([1, 2, 3]);
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Array(arr)) = toml_item {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array value");
    }
}

#[test]
fn test_json_object_to_inline_table() {
    let json_val = serde_json::json!({"key": "value"});
    let toml_item = json_to_toml_edit(&json_val).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::InlineTable(tbl)) = toml_item {
        assert!(tbl.contains_key("key"));
    } else {
        panic!("Expected inline table value");
    }
}

#[test]
fn test_round_trip_string() {
    let original = toml::Value::String("hello world".to_string());
    let json = toml_to_json(&original);
    let back = json_to_toml_value(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_round_trip_integer() {
    let original = toml::Value::Integer(12345);
    let json = toml_to_json(&original);
    let back = json_to_toml_value(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_round_trip_float() {
    let original = toml::Value::Float(2.718);
    let json = toml_to_json(&original);
    let back = json_to_toml_value(&json).unwrap();
    if let toml::Value::Float(f) = back {
        assert!((f - 2.718).abs() < f64::EPSILON);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn test_round_trip_boolean() {
    let original = toml::Value::Boolean(true);
    let json = toml_to_json(&original);
    let back = json_to_toml_value(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_round_trip_array() {
    let original = toml::Value::Array(vec![
        toml::Value::String("a".to_string()),
        toml::Value::String("b".to_string()),
    ]);
    let json = toml_to_json(&original);
    let back = json_to_toml_value(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_round_trip_nan() {
    let original = toml::Value::Float(f64::NAN);
    let json = toml_to_json(&original);
    assert_eq!(json, serde_json::Value::String("NaN".to_string()));
    let back = json_to_toml_value(&json).unwrap();
    if let toml::Value::Float(f) = back {
        assert!(f.is_nan());
    } else {
        panic!("Expected float NaN");
    }
}

#[test]
fn test_round_trip_infinity() {
    let original = toml::Value::Float(f64::INFINITY);
    let json = toml_to_json(&original);
    assert_eq!(json, serde_json::Value::String("Infinity".to_string()));
    let back = json_to_toml_value(&json).unwrap();
    if let toml::Value::Float(f) = back {
        assert!(f.is_infinite() && f.is_sign_positive());
    } else {
        panic!("Expected float Infinity");
    }
}

#[test]
fn test_preserving_type_converts_1_to_true_when_existing_is_bool() {
    let existing = toml_edit::value(true);
    let json_val = serde_json::json!(1);
    let result = json_to_toml_edit_preserving_type(&json_val, &existing).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Boolean(b)) = result {
        assert!(*b.value());
    } else {
        panic!("Expected boolean true, got {result:?}");
    }
}

#[test]
fn test_preserving_type_converts_0_to_false_when_existing_is_bool() {
    let existing = toml_edit::value(false);
    let json_val = serde_json::json!(0);
    let result = json_to_toml_edit_preserving_type(&json_val, &existing).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Boolean(b)) = result {
        assert!(!*b.value());
    } else {
        panic!("Expected boolean false, got {result:?}");
    }
}

#[test]
fn test_preserving_type_keeps_integer_when_existing_is_integer() {
    let existing = toml_edit::value(42);
    let json_val = serde_json::json!(1);
    let result = json_to_toml_edit_preserving_type(&json_val, &existing).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Integer(i)) = result {
        assert_eq!(*i.value(), 1);
    } else {
        panic!("Expected integer 1, got {result:?}");
    }
}

#[test]
fn test_preserving_type_keeps_bool_when_json_is_bool() {
    let existing = toml_edit::value(true);
    let json_val = serde_json::json!(false);
    let result = json_to_toml_edit_preserving_type(&json_val, &existing).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Boolean(b)) = result {
        assert!(!*b.value());
    } else {
        panic!("Expected boolean false, got {result:?}");
    }
}

#[test]
fn test_preserving_type_falls_back_for_non_01_integers() {
    let existing = toml_edit::value(true);
    let json_val = serde_json::json!(42);
    let result = json_to_toml_edit_preserving_type(&json_val, &existing).unwrap();
    if let toml_edit::Item::Value(toml_edit::Value::Integer(i)) = result {
        assert_eq!(*i.value(), 42);
    } else {
        panic!("Expected integer 42, got {result:?}");
    }
}
