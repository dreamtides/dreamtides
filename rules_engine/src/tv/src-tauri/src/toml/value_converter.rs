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
