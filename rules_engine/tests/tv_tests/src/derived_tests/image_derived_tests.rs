use std::collections::HashMap;
use std::sync::Arc;

use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::image_derived::ImageDerivedFunction;
use tv_lib::images::image_cache::ImageCache;

fn create_cache() -> (tempfile::TempDir, Arc<ImageCache>) {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    (temp, cache)
}

fn empty_context() -> LookupContext {
    LookupContext::new()
}

#[test]
fn test_image_derived_function_name() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    assert_eq!(function.name(), "image_derived");
}

#[test]
fn test_image_derived_input_keys() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    assert_eq!(function.input_keys(), vec!["image_number"]);
}

#[test]
fn test_image_derived_is_async() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    assert!(function.is_async(), "ImageDerivedFunction should be async");
}

#[test]
fn test_image_derived_empty_string_returns_empty_text() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(""));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_derived_null_returns_empty_text() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::Value::Null);

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_derived_missing_field_returns_empty_text() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let inputs: RowData = HashMap::new();

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_derived_invalid_type_returns_error() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(["array", "value"]));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Invalid image_number type"),
                "Error should mention invalid type: {msg}"
            );
            assert!(msg.contains("array"), "Error should mention the actual type: {msg}");
        }
        _ => panic!("Expected error result for invalid type, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_boolean_type_returns_error() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(true));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Invalid image_number type"),
                "Error should mention invalid type: {msg}"
            );
            assert!(msg.contains("boolean"), "Error should mention boolean type: {msg}");
        }
        _ => panic!("Expected error result for boolean type, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_object_type_returns_error() {
    let (_temp, cache) = create_cache();
    let function = ImageDerivedFunction::new(cache);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!({"key": "value"}));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Invalid image_number type"),
                "Error should mention invalid type: {msg}"
            );
            assert!(msg.contains("object"), "Error should mention object type: {msg}");
        }
        _ => panic!("Expected error result for object type, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_cache_hit_returns_path() {
    let (_temp, cache) = create_cache();
    let template = "https://example.com/images/{image_number}.png";
    let function = ImageDerivedFunction::with_template(Arc::clone(&cache), template);
    let context = empty_context();

    let url = "https://example.com/images/42.png";
    let fake_image_data = vec![0xFF; 100];
    let cached_path = cache.put(url, &fake_image_data).expect("Should store in cache");

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("42"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Image(path) => {
            assert_eq!(
                path,
                cached_path.to_string_lossy().to_string(),
                "Should return cached file path"
            );
        }
        _ => panic!("Expected Image result for cache hit, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_cache_hit_with_numeric_input() {
    let (_temp, cache) = create_cache();
    let template = "https://example.com/images/{image_number}.png";
    let function = ImageDerivedFunction::with_template(Arc::clone(&cache), template);
    let context = empty_context();

    let url = "https://example.com/images/99.png";
    let fake_image_data = vec![0xAB; 50];
    let cached_path = cache.put(url, &fake_image_data).expect("Should store in cache");

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(99));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Image(path) => {
            assert_eq!(
                path,
                cached_path.to_string_lossy().to_string(),
                "Should return cached file path for numeric input"
            );
        }
        _ => panic!("Expected Image result for numeric cache hit, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_custom_template() {
    let (_temp, cache) = create_cache();
    let template = "https://cdn.example.com/cards/{image_number}.jpg";
    let function = ImageDerivedFunction::with_template(Arc::clone(&cache), template);
    let context = empty_context();

    let url = "https://cdn.example.com/cards/abc-123.jpg";
    let fake_image_data = vec![0xCD; 75];
    let cached_path = cache.put(url, &fake_image_data).expect("Should store in cache");

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("abc-123"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Image(path) => {
            assert_eq!(
                path,
                cached_path.to_string_lossy().to_string(),
                "Should return cached path with custom template"
            );
        }
        _ => panic!("Expected Image result, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_cache_miss_returns_error_for_unreachable_url() {
    let (_temp, cache) = create_cache();
    let template = "https://invalid-host-that-does-not-exist.example.invalid/{image_number}.png";
    let function = ImageDerivedFunction::with_template(cache, template);
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("1"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Image fetch failed"),
                "Error should mention fetch failure: {msg}"
            );
        }
        _ => panic!("Expected error for unreachable URL, got: {result:?}"),
    }
}

#[test]
fn test_image_derived_multiple_cache_hits() {
    let (_temp, cache) = create_cache();
    let template = "https://example.com/{image_number}.png";
    let function = ImageDerivedFunction::with_template(Arc::clone(&cache), template);
    let context = empty_context();

    let url1 = "https://example.com/100.png";
    let url2 = "https://example.com/200.png";
    cache.put(url1, &vec![0x01; 50]).expect("Should store first");
    let path2 = cache.put(url2, &vec![0x02; 50]).expect("Should store second");

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("200"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Image(path) => {
            assert_eq!(path, path2.to_string_lossy().to_string());
        }
        _ => panic!("Expected Image result, got: {result:?}"),
    }
}
