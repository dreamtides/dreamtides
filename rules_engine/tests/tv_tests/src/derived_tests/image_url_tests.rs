use std::collections::HashMap;

use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::image_url::ImageUrlFunction;

fn empty_context() -> LookupContext {
    LookupContext::new()
}

#[test]
fn test_image_url_function_name() {
    let function = ImageUrlFunction::new();
    assert_eq!(function.name(), "image_url");
}

#[test]
fn test_image_url_input_keys() {
    let function = ImageUrlFunction::new();
    assert_eq!(function.input_keys(), vec!["image_number"]);
}

#[test]
fn test_image_url_is_not_async() {
    let function = ImageUrlFunction::new();
    assert!(!function.is_async(), "ImageUrlFunction should be synchronous");
}

#[test]
fn test_image_url_default_template_with_string() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("12345"));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/12345.png".to_string())
    );
}

#[test]
fn test_image_url_default_template_with_number() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(67890));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/67890.png".to_string())
    );
}

#[test]
fn test_image_url_custom_template() {
    let function =
        ImageUrlFunction::with_template("https://cdn.example.com/images/{image_number}.jpg");
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("abc-123"));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://cdn.example.com/images/abc-123.jpg".to_string())
    );
}

#[test]
fn test_image_url_custom_template_with_number() {
    let function =
        ImageUrlFunction::with_template("https://cdn.example.com/{image_number}/card.png");
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(42));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Image("https://cdn.example.com/42/card.png".to_string()));
}

#[test]
fn test_image_url_empty_string_returns_empty_text() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(""));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_url_null_returns_empty_text() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::Value::Null);

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_url_missing_field_returns_empty_text() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let inputs: RowData = HashMap::new();

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_image_url_array_type_returns_error() {
    let function = ImageUrlFunction::new();
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
        _ => panic!("Expected error result for array type, got: {result:?}"),
    }
}

#[test]
fn test_image_url_boolean_type_returns_error() {
    let function = ImageUrlFunction::new();
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
fn test_image_url_object_type_returns_error() {
    let function = ImageUrlFunction::new();
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
fn test_image_url_default_constructor() {
    let function = ImageUrlFunction::default();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("999"));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/999.png".to_string())
    );
}

#[test]
fn test_image_url_template_with_multiple_placeholders() {
    let function =
        ImageUrlFunction::with_template("https://example.com/{image_number}/{image_number}.png");
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("42"));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Image("https://example.com/42/42.png".to_string()));
}

#[test]
fn test_image_url_template_no_placeholder() {
    let function = ImageUrlFunction::with_template("https://example.com/static.png");
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("12345"));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Image("https://example.com/static.png".to_string()));
}

#[test]
fn test_image_url_numeric_float_input() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(3.14));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/3.14.png".to_string())
    );
}

#[test]
fn test_image_url_extra_inputs_ignored() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("500"));
    inputs.insert("name".to_string(), serde_json::json!("Card Name"));
    inputs.insert("cost".to_string(), serde_json::json!(5));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/500.png".to_string())
    );
}

#[test]
fn test_image_url_special_characters_in_input() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!("hello world"));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image(
            "https://dreamtides-assets.example.com/cards/hello world.png".to_string()
        )
    );
}

#[test]
fn test_image_url_negative_number() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(-1));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/-1.png".to_string())
    );
}

#[test]
fn test_image_url_zero_number() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(0));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/0.png".to_string())
    );
}

#[test]
fn test_image_url_large_number() {
    let function = ImageUrlFunction::new();
    let context = empty_context();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("image_number".to_string(), serde_json::json!(9999999));

    let result = function.compute(&inputs, &context);
    assert_eq!(
        result,
        DerivedResult::Image("https://dreamtides-assets.example.com/cards/9999999.png".to_string())
    );
}
