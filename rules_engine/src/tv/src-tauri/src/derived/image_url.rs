use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};

/// Default URL template for image lookup.
/// Uses `{image_number}` as placeholder for the image identifier.
const DEFAULT_IMAGE_URL_TEMPLATE: &str =
    "https://dreamtides-assets.example.com/cards/{image_number}.png";

/// A derived function that constructs image URLs from row data.
///
/// Given a cell value containing an image number, this function constructs
/// a web URL using a configurable template. The URL can then be used by the
/// frontend to display the image in a cell.
pub struct ImageUrlFunction {
    url_template: String,
}

impl ImageUrlFunction {
    /// Creates a new ImageUrlFunction with the default URL template.
    pub fn new() -> Self {
        Self { url_template: DEFAULT_IMAGE_URL_TEMPLATE.to_string() }
    }

    /// Creates a new ImageUrlFunction with a custom URL template.
    ///
    /// The template should contain `{image_number}` as a placeholder that
    /// will be replaced with the actual image number from the row data.
    pub fn with_template(template: impl Into<String>) -> Self {
        Self { url_template: template.into() }
    }

    fn construct_url(&self, image_number: &str) -> String {
        self.url_template.replace("{image_number}", image_number)
    }
}

impl Default for ImageUrlFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivedFunction for ImageUrlFunction {
    fn name(&self) -> &'static str {
        "image_url"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["image_number"]
    }

    fn compute(&self, inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        let image_number = match inputs.get("image_number") {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Number(n)) => {
                return DerivedResult::Image(self.construct_url(&n.to_string()));
            }
            Some(serde_json::Value::Null) | None => {
                return DerivedResult::Text(String::new());
            }
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid image_number type: expected string or number, got {}",
                    json_type_name(other)
                ));
            }
        };

        if image_number.is_empty() {
            return DerivedResult::Text(String::new());
        }

        DerivedResult::Image(self.construct_url(image_number))
    }

    fn is_async(&self) -> bool {
        false
    }
}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn create_empty_context() -> LookupContext {
        LookupContext::new()
    }

    #[test]
    fn test_construct_url_with_string_number() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::json!("12345"));

        let result = function.compute(&inputs, &context);
        assert_eq!(
            result,
            DerivedResult::Image(
                "https://dreamtides-assets.example.com/cards/12345.png".to_string()
            )
        );
    }

    #[test]
    fn test_construct_url_with_numeric_value() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::json!(67890));

        let result = function.compute(&inputs, &context);
        assert_eq!(
            result,
            DerivedResult::Image(
                "https://dreamtides-assets.example.com/cards/67890.png".to_string()
            )
        );
    }

    #[test]
    fn test_custom_url_template() {
        let function =
            ImageUrlFunction::with_template("https://cdn.example.com/images/{image_number}.jpg");
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::json!("abc-123"));

        let result = function.compute(&inputs, &context);
        assert_eq!(
            result,
            DerivedResult::Image("https://cdn.example.com/images/abc-123.jpg".to_string())
        );
    }

    #[test]
    fn test_empty_image_number() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::json!(""));

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_null_image_number() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::Value::Null);

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_missing_image_number_field() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let inputs: RowData = HashMap::new();

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_invalid_type() {
        let function = ImageUrlFunction::new();
        let context = create_empty_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("image_number".to_string(), serde_json::json!(["array", "value"]));

        let result = function.compute(&inputs, &context);
        match result {
            DerivedResult::Error(msg) => {
                assert!(
                    msg.contains("Invalid image_number type"),
                    "Error should mention invalid type: {msg}"
                );
            }
            _ => panic!("Expected error result, got: {result:?}"),
        }
    }

    #[test]
    fn test_function_name() {
        let function = ImageUrlFunction::new();
        assert_eq!(function.name(), "image_url");
    }

    #[test]
    fn test_input_keys() {
        let function = ImageUrlFunction::new();
        assert_eq!(function.input_keys(), vec!["image_number"]);
    }

    #[test]
    fn test_is_not_async() {
        let function = ImageUrlFunction::new();
        assert!(!function.is_async());
    }
}
