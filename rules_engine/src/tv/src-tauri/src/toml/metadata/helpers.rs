use crate::error::error_types::TvError;

/// Converts a TOML value to a serde_json value for JSON serialization.
pub fn toml_value_to_json(val: &toml::Value) -> serde_json::Value {
    match val {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::json!(*i),
        toml::Value::Float(f) => serde_json::json!(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(t) => {
            let map: serde_json::Map<String, serde_json::Value> =
                t.iter().map(|(k, v)| (k.clone(), toml_value_to_json(v))).collect();
            serde_json::Value::Object(map)
        }
    }
}

/// Parses TOML content and returns a Result with a properly formatted error.
pub fn parse_toml_content(content: &str, file_path: &str) -> Result<toml::Value, TvError> {
    toml::from_str(content).map_err(|e| TvError::TomlParseError {
        path: file_path.to_string(),
        line: e.span().map(|s| content[..s.start].lines().count()),
        message: e.message().to_string(),
    })
}
