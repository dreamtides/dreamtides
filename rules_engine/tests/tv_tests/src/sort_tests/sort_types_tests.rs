use std::cmp::Ordering;

use tv_lib::sort::sort_types::{CellValue, SortDirection, SortState};

#[test]
fn test_sort_direction_default_is_ascending() {
    let direction = SortDirection::default();
    assert_eq!(direction, SortDirection::Ascending);
}

#[test]
fn test_sort_direction_toggle_ascending_to_descending() {
    assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
}

#[test]
fn test_sort_direction_toggle_descending_to_ascending() {
    assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
}

#[test]
fn test_sort_direction_double_toggle_is_identity() {
    assert_eq!(SortDirection::Ascending.toggle().toggle(), SortDirection::Ascending);
    assert_eq!(SortDirection::Descending.toggle().toggle(), SortDirection::Descending);
}

#[test]
fn test_sort_direction_clone() {
    let d = SortDirection::Ascending;
    let cloned = d;
    assert_eq!(d, cloned);
}

#[test]
fn test_sort_direction_debug_format() {
    assert_eq!(format!("{:?}", SortDirection::Ascending), "Ascending");
    assert_eq!(format!("{:?}", SortDirection::Descending), "Descending");
}

#[test]
fn test_sort_direction_serialize_ascending() {
    let json = serde_json::to_string(&SortDirection::Ascending).unwrap();
    assert_eq!(json, "\"ascending\"");
}

#[test]
fn test_sort_direction_serialize_descending() {
    let json = serde_json::to_string(&SortDirection::Descending).unwrap();
    assert_eq!(json, "\"descending\"");
}

#[test]
fn test_sort_direction_deserialize_ascending() {
    let d: SortDirection = serde_json::from_str("\"ascending\"").unwrap();
    assert_eq!(d, SortDirection::Ascending);
}

#[test]
fn test_sort_direction_deserialize_descending() {
    let d: SortDirection = serde_json::from_str("\"descending\"").unwrap();
    assert_eq!(d, SortDirection::Descending);
}

#[test]
fn test_sort_direction_roundtrip_serialization() {
    for direction in [SortDirection::Ascending, SortDirection::Descending] {
        let json = serde_json::to_string(&direction).unwrap();
        let deserialized: SortDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(direction, deserialized);
    }
}

#[test]
fn test_sort_state_new() {
    let state = SortState::new("cost".to_string(), SortDirection::Descending);
    assert_eq!(state.column, "cost");
    assert_eq!(state.direction, SortDirection::Descending);
}

#[test]
fn test_sort_state_ascending_constructor() {
    let state = SortState::ascending("name".to_string());
    assert_eq!(state.column, "name");
    assert_eq!(state.direction, SortDirection::Ascending);
}

#[test]
fn test_sort_state_descending_constructor() {
    let state = SortState::descending("cost".to_string());
    assert_eq!(state.column, "cost");
    assert_eq!(state.direction, SortDirection::Descending);
}

#[test]
fn test_sort_state_clone() {
    let state = SortState::ascending("name".to_string());
    let cloned = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn test_sort_state_debug_format() {
    let state = SortState::ascending("name".to_string());
    let debug = format!("{state:?}");
    assert!(debug.contains("name"));
    assert!(debug.contains("Ascending"));
}

#[test]
fn test_sort_state_serialize() {
    let state = SortState::ascending("name".to_string());
    let json = serde_json::to_value(&state).unwrap();
    assert_eq!(json["column"], "name");
    assert_eq!(json["direction"], "ascending");
}

#[test]
fn test_sort_state_deserialize() {
    let json = r#"{"column":"cost","direction":"descending"}"#;
    let state: SortState = serde_json::from_str(json).unwrap();
    assert_eq!(state.column, "cost");
    assert_eq!(state.direction, SortDirection::Descending);
}

#[test]
fn test_sort_state_roundtrip_serialization() {
    let state = SortState::descending("spark".to_string());
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: SortState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, deserialized);
}

#[test]
fn test_sort_state_equality() {
    let a = SortState::ascending("name".to_string());
    let b = SortState::ascending("name".to_string());
    assert_eq!(a, b);
}

#[test]
fn test_sort_state_inequality_column() {
    let a = SortState::ascending("name".to_string());
    let b = SortState::ascending("cost".to_string());
    assert_ne!(a, b);
}

#[test]
fn test_sort_state_inequality_direction() {
    let a = SortState::ascending("name".to_string());
    let b = SortState::descending("name".to_string());
    assert_ne!(a, b);
}

#[test]
fn test_cell_value_from_json_null() {
    let val = CellValue::from_json(&serde_json::json!(null));
    assert_eq!(val, CellValue::Null);
}

#[test]
fn test_cell_value_from_json_bool_true() {
    let val = CellValue::from_json(&serde_json::json!(true));
    assert_eq!(val, CellValue::Boolean(true));
}

#[test]
fn test_cell_value_from_json_bool_false() {
    let val = CellValue::from_json(&serde_json::json!(false));
    assert_eq!(val, CellValue::Boolean(false));
}

#[test]
fn test_cell_value_from_json_integer() {
    let val = CellValue::from_json(&serde_json::json!(42));
    assert_eq!(val, CellValue::Integer(42));
}

#[test]
fn test_cell_value_from_json_negative_integer() {
    let val = CellValue::from_json(&serde_json::json!(-7));
    assert_eq!(val, CellValue::Integer(-7));
}

#[test]
fn test_cell_value_from_json_zero() {
    let val = CellValue::from_json(&serde_json::json!(0));
    assert_eq!(val, CellValue::Integer(0));
}

#[test]
fn test_cell_value_from_json_float() {
    let val = CellValue::from_json(&serde_json::json!(3.14));
    assert_eq!(val, CellValue::Float(3.14));
}

#[test]
fn test_cell_value_from_json_negative_float() {
    let val = CellValue::from_json(&serde_json::json!(-2.5));
    assert_eq!(val, CellValue::Float(-2.5));
}

#[test]
fn test_cell_value_from_json_string() {
    let val = CellValue::from_json(&serde_json::json!("hello"));
    assert_eq!(val, CellValue::String("hello".to_string()));
}

#[test]
fn test_cell_value_from_json_empty_string() {
    let val = CellValue::from_json(&serde_json::json!(""));
    assert_eq!(val, CellValue::String(String::new()));
}

#[test]
fn test_cell_value_from_json_unicode_string() {
    let val = CellValue::from_json(&serde_json::json!("日本語テスト"));
    assert_eq!(val, CellValue::String("日本語テスト".to_string()));
}

#[test]
fn test_cell_value_from_json_array() {
    let val = CellValue::from_json(&serde_json::json!([1, 2, 3]));
    match val {
        CellValue::String(_) => {}
        other => panic!("Expected String variant for array, got {other:?}"),
    }
}

#[test]
fn test_cell_value_from_json_object() {
    let val = CellValue::from_json(&serde_json::json!({"key": "value"}));
    match val {
        CellValue::String(_) => {}
        other => panic!("Expected String variant for object, got {other:?}"),
    }
}

#[test]
fn test_cell_value_cmp_null_null() {
    assert_eq!(CellValue::Null.cmp_values(&CellValue::Null), Ordering::Equal);
}

#[test]
fn test_cell_value_cmp_null_sorts_last() {
    assert_eq!(CellValue::Null.cmp_values(&CellValue::Integer(1)), Ordering::Greater);
    assert_eq!(CellValue::Integer(1).cmp_values(&CellValue::Null), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_null_after_string() {
    assert_eq!(CellValue::Null.cmp_values(&CellValue::String("z".to_string())), Ordering::Greater);
}

#[test]
fn test_cell_value_cmp_null_after_boolean() {
    assert_eq!(CellValue::Null.cmp_values(&CellValue::Boolean(false)), Ordering::Greater);
}

#[test]
fn test_cell_value_cmp_boolean_false_before_true() {
    assert_eq!(CellValue::Boolean(false).cmp_values(&CellValue::Boolean(true)), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_boolean_equal() {
    assert_eq!(CellValue::Boolean(true).cmp_values(&CellValue::Boolean(true)), Ordering::Equal);
}

#[test]
fn test_cell_value_cmp_integer_ascending() {
    assert_eq!(CellValue::Integer(1).cmp_values(&CellValue::Integer(5)), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_integer_equal() {
    assert_eq!(CellValue::Integer(42).cmp_values(&CellValue::Integer(42)), Ordering::Equal);
}

#[test]
fn test_cell_value_cmp_integer_descending() {
    assert_eq!(CellValue::Integer(10).cmp_values(&CellValue::Integer(3)), Ordering::Greater);
}

#[test]
fn test_cell_value_cmp_negative_integers() {
    assert_eq!(CellValue::Integer(-5).cmp_values(&CellValue::Integer(-3)), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_float_ascending() {
    assert_eq!(CellValue::Float(1.5).cmp_values(&CellValue::Float(2.5)), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_float_equal() {
    assert_eq!(CellValue::Float(3.14).cmp_values(&CellValue::Float(3.14)), Ordering::Equal);
}

#[test]
fn test_cell_value_cmp_integer_vs_float() {
    assert_eq!(CellValue::Integer(5).cmp_values(&CellValue::Float(3.7)), Ordering::Greater);
}

#[test]
fn test_cell_value_cmp_float_vs_integer() {
    assert_eq!(CellValue::Float(3.7).cmp_values(&CellValue::Integer(5)), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_integer_equals_float() {
    assert_eq!(CellValue::Integer(3).cmp_values(&CellValue::Float(3.0)), Ordering::Equal);
}

#[test]
fn test_cell_value_cmp_string_case_insensitive() {
    assert_eq!(
        CellValue::String("apple".to_string()).cmp_values(&CellValue::String("Banana".to_string())),
        Ordering::Less
    );
}

#[test]
fn test_cell_value_cmp_string_equal_case_insensitive() {
    assert_eq!(
        CellValue::String("Hello".to_string()).cmp_values(&CellValue::String("hello".to_string())),
        Ordering::Equal
    );
}

#[test]
fn test_cell_value_cmp_string_alphabetical() {
    assert_eq!(
        CellValue::String("abc".to_string()).cmp_values(&CellValue::String("xyz".to_string())),
        Ordering::Less
    );
}

#[test]
fn test_cell_value_cmp_empty_string() {
    assert_eq!(
        CellValue::String(String::new()).cmp_values(&CellValue::String("a".to_string())),
        Ordering::Less
    );
}

#[test]
fn test_cell_value_cmp_cross_type_boolean_before_integer() {
    let bool_val = CellValue::Boolean(true);
    let int_val = CellValue::Integer(1);
    assert_eq!(bool_val.cmp_values(&int_val), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_cross_type_integer_before_string() {
    let int_val = CellValue::Integer(1);
    let str_val = CellValue::String("a".to_string());
    assert_eq!(int_val.cmp_values(&str_val), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_cross_type_float_before_string() {
    let float_val = CellValue::Float(1.0);
    let str_val = CellValue::String("a".to_string());
    assert_eq!(float_val.cmp_values(&str_val), Ordering::Less);
}

#[test]
fn test_cell_value_cmp_cross_type_boolean_before_string() {
    let bool_val = CellValue::Boolean(false);
    let str_val = CellValue::String("a".to_string());
    assert_eq!(bool_val.cmp_values(&str_val), Ordering::Less);
}

#[test]
fn test_cell_value_partial_ord_consistent_with_cmp_values() {
    let a = CellValue::Integer(5);
    let b = CellValue::Integer(10);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    assert_eq!(a.cmp_values(&b), Ordering::Less);
}

#[test]
fn test_cell_value_partial_ord_for_strings() {
    let a = CellValue::String("apple".to_string());
    let b = CellValue::String("banana".to_string());
    assert!(a < b);
}

#[test]
fn test_cell_value_clone() {
    let val = CellValue::String("test".to_string());
    let cloned = val.clone();
    assert_eq!(val, cloned);
}

#[test]
fn test_cell_value_debug_format() {
    assert!(format!("{:?}", CellValue::Null).contains("Null"));
    assert!(format!("{:?}", CellValue::Boolean(true)).contains("true"));
    assert!(format!("{:?}", CellValue::Integer(42)).contains("42"));
    assert!(format!("{:?}", CellValue::Float(3.14)).contains("3.14"));
    assert!(format!("{:?}", CellValue::String("hi".to_string())).contains("hi"));
}

#[test]
fn test_cell_value_from_json_large_integer() {
    let val = CellValue::from_json(&serde_json::json!(i64::MAX));
    assert_eq!(val, CellValue::Integer(i64::MAX));
}

#[test]
fn test_cell_value_cmp_large_integers() {
    assert_eq!(
        CellValue::Integer(i64::MAX).cmp_values(&CellValue::Integer(i64::MIN)),
        Ordering::Greater
    );
}

#[test]
fn test_cell_value_type_ordering() {
    let values = vec![
        CellValue::Null,
        CellValue::String("z".to_string()),
        CellValue::Integer(99),
        CellValue::Boolean(false),
    ];

    let mut sorted: Vec<CellValue> = values;
    sorted.sort_by(|a, b| a.cmp_values(b));

    assert_eq!(sorted[0], CellValue::Boolean(false));
    assert_eq!(sorted[1], CellValue::Integer(99));
    assert_eq!(sorted[2], CellValue::String("z".to_string()));
    assert_eq!(sorted[3], CellValue::Null);
}
