use std::io::Cursor;
use std::sync::Arc;

use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use crate::images::image_cache::ImageCache;

/// Default URL template for image lookup.
/// Uses `{image_number}` as placeholder for the image identifier.
const DEFAULT_IMAGE_URL_TEMPLATE: &str =
    "https://dreamtides-assets.example.com/cards/{image_number}.png";

/// Default timeout for HTTP image fetches in seconds.
const DEFAULT_FETCH_TIMEOUT_SECS: u64 = 30;

/// A derived function that fetches images and returns local cache file paths.
///
/// Given a cell value containing an image number, this function constructs
/// a URL, checks the image cache, fetches on cache miss, and returns the
/// local cache file path. The frontend converts this path to an asset URL.
pub struct ImageDerivedFunction {
    url_template: String,
    cache: Arc<ImageCache>,
    client: reqwest::blocking::Client,
}

impl ImageDerivedFunction {
    /// Creates a new ImageDerivedFunction with the default URL template.
    pub fn new(cache: Arc<ImageCache>) -> Self {
        Self::with_template(cache, DEFAULT_IMAGE_URL_TEMPLATE)
    }

    /// Creates a new ImageDerivedFunction with a custom URL template.
    pub fn with_template(cache: Arc<ImageCache>, template: impl Into<String>) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_FETCH_TIMEOUT_SECS))
            .build()
            .expect("Failed to create blocking HTTP client");
        Self { url_template: template.into(), cache, client }
    }

    fn construct_url(&self, template: &str, image_number: &str) -> String {
        template.replace("{image_number}", image_number)
    }

    fn fetch_and_cache(&self, url: &str) -> DerivedResult {
        // Check cache first
        if let Some(cached_path) = self.cache.get(url) {
            tracing::debug!(
                component = "tv.derived.image",
                url = %url,
                "Image cache hit"
            );
            return DerivedResult::Image(cached_path.to_string_lossy().to_string());
        }

        // Fetch from remote URL
        tracing::info!(
            component = "tv.derived.image",
            url = %url,
            "Fetching image (cache miss)"
        );

        let response = match self.client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => {
                let message = if e.is_timeout() {
                    "Request timed out".to_string()
                } else if e.is_connect() {
                    "Connection failed".to_string()
                } else {
                    format!("Network error: {e}")
                };
                return DerivedResult::Error(format!("Image fetch failed for {url}: {message}"));
            }
        };

        let status = response.status();
        if !status.is_success() {
            return DerivedResult::Error(format!(
                "Image fetch failed for {url}: HTTP {status}"
            ));
        }

        let bytes = match response.bytes() {
            Ok(b) => b,
            Err(e) => {
                return DerivedResult::Error(format!(
                    "Failed to read image response for {url}: {e}"
                ));
            }
        };

        // Validate image format
        if let Err(e) = validate_image_data(&bytes) {
            return DerivedResult::Error(format!(
                "Invalid image data from {url}: {e}"
            ));
        }

        // Store in cache
        match self.cache.put(url, &bytes) {
            Ok(cached_path) => {
                tracing::info!(
                    component = "tv.derived.image",
                    url = %url,
                    size = bytes.len(),
                    "Image fetched and cached"
                );
                DerivedResult::Image(cached_path.to_string_lossy().to_string())
            }
            Err(e) => DerivedResult::Error(format!(
                "Failed to cache image from {url}: {e}"
            )),
        }
    }
}

impl DerivedFunction for ImageDerivedFunction {
    fn name(&self) -> &'static str {
        "image_derived"
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
                let url = self.construct_url(template, &n.to_string());
                return self.fetch_and_cache(&url);
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

        let url = self.construct_url(template, image_number);
        self.fetch_and_cache(&url)
    }

    fn is_async(&self) -> bool {
        true
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

fn validate_image_data(data: &[u8]) -> Result<(), String> {
    let cursor = Cursor::new(data);
    image::ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| format!("Failed to detect image format: {e}"))?
        .format()
        .ok_or_else(|| "Unrecognized image format".to_string())?;
    Ok(())
}
