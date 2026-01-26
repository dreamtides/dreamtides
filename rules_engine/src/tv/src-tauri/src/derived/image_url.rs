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

    fn construct_url(template: &str, image_number: &str) -> String {
        template.replace("{image_number}", image_number)
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
        let template = inputs
            .get("__url_template")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.url_template);

        let image_value = inputs
            .get("image-number")
            .or_else(|| inputs.get("image_number"));

        let image_number = match image_value {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Number(n)) => {
                return DerivedResult::Image(Self::construct_url(template, &n.to_string()));
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

        DerivedResult::Image(Self::construct_url(template, image_number))
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
