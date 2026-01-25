/// Converts a TOML value to a JSON value for the frontend.
pub fn toml_to_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => float_to_json(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(toml_to_json).collect()),
        toml::Value::Table(tbl) => {
            let map: serde_json::Map<String, serde_json::Value> =
                tbl.iter().map(|(k, v)| (k.clone(), toml_to_json(v))).collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

/// Converts a JSON value to a toml_edit Item for preserving document formatting.
pub fn json_to_toml_edit(value: &serde_json::Value) -> Option<toml_edit::Item> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(toml_edit::value(*b)),
        serde_json::Value::Number(n) => convert_json_number_to_toml(n),
        serde_json::Value::String(s) => Some(convert_json_string_to_toml(s)),
        serde_json::Value::Array(arr) => Some(convert_json_array_to_toml(arr)),
        serde_json::Value::Object(obj) => Some(convert_json_object_to_inline_table(obj)),
    }
}

/// Converts a JSON value to a toml_edit Item, preserving the type of an existing
/// TOML value when possible.
///
/// This is important for round-trip preservation: spreadsheet libraries like Univer
/// may convert boolean values to integers (true -> 1, false -> 0). When the original
/// TOML value was a boolean, we convert numeric 0/1 back to boolean false/true.
pub fn json_to_toml_edit_preserving_type(
    value: &serde_json::Value,
    existing: &toml_edit::Item,
) -> Option<toml_edit::Item> {
    // If the existing value is a boolean and the new value is a number (0 or 1),
    // convert the number back to boolean to preserve the original type.
    if let toml_edit::Item::Value(toml_edit::Value::Boolean(_)) = existing {
        if let serde_json::Value::Number(n) = value {
            if let Some(i) = n.as_i64() {
                if i == 0 {
                    return Some(toml_edit::value(false));
                } else if i == 1 {
                    return Some(toml_edit::value(true));
                }
            }
        }
    }

    // Fall back to standard conversion
    json_to_toml_edit(value)
}

/// Converts a JSON value to a toml::Value for general use.
pub fn json_to_toml_value(value: &serde_json::Value) -> Option<toml::Value> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            n.as_i64().map(toml::Value::Integer).or_else(|| n.as_f64().map(toml::Value::Float))
        }
        serde_json::Value::String(s) => {
            if let Some(special_val) = parse_special_string_value(s) {
                Some(special_val)
            } else {
                Some(toml::Value::String(s.clone()))
            }
        }
        serde_json::Value::Array(arr) => {
            let values: Vec<toml::Value> = arr.iter().filter_map(json_to_toml_value).collect();
            Some(toml::Value::Array(values))
        }
        serde_json::Value::Object(obj) => {
            let table: toml::map::Map<String, toml::Value> = obj
                .iter()
                .filter_map(|(k, v)| json_to_toml_value(v).map(|tv| (k.clone(), tv)))
                .collect();
            Some(toml::Value::Table(table))
        }
    }
}

fn convert_json_number_to_toml(n: &serde_json::Number) -> Option<toml_edit::Item> {
    n.as_i64().map(toml_edit::value).or_else(|| n.as_f64().map(toml_edit::value))
}

fn convert_json_string_to_toml(s: &str) -> toml_edit::Item {
    if let Some(special_val) = parse_special_string_as_toml_edit(s) {
        return special_val;
    }
    toml_edit::value(s)
}

fn convert_json_array_to_toml(arr: &[serde_json::Value]) -> toml_edit::Item {
    let mut toml_arr = toml_edit::Array::new();
    for item in arr {
        if let Some(toml_edit::Item::Value(v)) = json_to_toml_edit(item) {
            toml_arr.push(v);
        }
    }
    toml_edit::Item::Value(toml_edit::Value::Array(toml_arr))
}

fn convert_json_object_to_inline_table(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> toml_edit::Item {
    let mut inline_table = toml_edit::InlineTable::new();
    for (key, val) in obj {
        if let Some(toml_edit::Item::Value(v)) = json_to_toml_edit(val) {
            inline_table.insert(key, v);
        }
    }
    toml_edit::Item::Value(toml_edit::Value::InlineTable(inline_table))
}

fn parse_special_string_value(s: &str) -> Option<toml::Value> {
    match s {
        "NaN" => Some(toml::Value::Float(f64::NAN)),
        "Infinity" => Some(toml::Value::Float(f64::INFINITY)),
        "-Infinity" => Some(toml::Value::Float(f64::NEG_INFINITY)),
        _ => None,
    }
}

fn parse_special_string_as_toml_edit(s: &str) -> Option<toml_edit::Item> {
    match s {
        "NaN" => Some(toml_edit::value(f64::NAN)),
        "Infinity" => Some(toml_edit::value(f64::INFINITY)),
        "-Infinity" => Some(toml_edit::value(f64::NEG_INFINITY)),
        _ => None,
    }
}

fn float_to_json(f: f64) -> serde_json::Value {
    if f.is_nan() {
        serde_json::Value::String("NaN".to_string())
    } else if f.is_infinite() {
        if f.is_sign_positive() {
            serde_json::Value::String("Infinity".to_string())
        } else {
            serde_json::Value::String("-Infinity".to_string())
        }
    } else {
        serde_json::Number::from_f64(f).map_or(serde_json::Value::Null, serde_json::Value::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            panic!("Expected boolean true, got {:?}", result);
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
            panic!("Expected boolean false, got {:?}", result);
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
            panic!("Expected integer 1, got {:?}", result);
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
            panic!("Expected boolean false, got {:?}", result);
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
            panic!("Expected integer 42, got {:?}", result);
        }
    }
}
