use std::collections::HashMap;

use tv_lib::derived::derived_types::{
    DerivedFunction, DerivedResult, LookupContext, RowData, StyledSpan,
};
use tv_lib::derived::function_registry::FunctionRegistry;

struct TestFunction {
    result: DerivedResult,
}

impl TestFunction {
    fn new(result: DerivedResult) -> Self {
        Self { result }
    }
}

impl DerivedFunction for TestFunction {
    fn name(&self) -> &'static str {
        "test_function"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["input"]
    }

    fn compute(&self, _inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        self.result.clone()
    }
}

struct AsyncTestFunction;

impl DerivedFunction for AsyncTestFunction {
    fn name(&self) -> &'static str {
        "async_test"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["input"]
    }

    fn compute(&self, _inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        DerivedResult::Text("async result".to_string())
    }

    fn is_async(&self) -> bool {
        true
    }
}

#[test]
fn test_derived_result_text_variant() {
    let result = DerivedResult::Text("hello".to_string());
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "text", "Text variant should have type 'text'");
    assert_eq!(json["value"], "hello", "Text variant value mismatch");
}

#[test]
fn test_derived_result_number_variant() {
    let result = DerivedResult::Number(42.5);
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "number", "Number variant should have type 'number'");
    assert_eq!(json["value"], 42.5, "Number variant value mismatch");
}

#[test]
fn test_derived_result_boolean_variant() {
    let result = DerivedResult::Boolean(true);
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "boolean", "Boolean variant should have type 'boolean'");
    assert_eq!(json["value"], true, "Boolean variant value mismatch");
}

#[test]
fn test_derived_result_image_variant() {
    let result = DerivedResult::Image("https://example.com/img.png".to_string());
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "image", "Image variant should have type 'image'");
    assert_eq!(json["value"], "https://example.com/img.png", "Image variant value mismatch");
}

#[test]
fn test_derived_result_error_variant() {
    let result = DerivedResult::Error("something failed".to_string());
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "error", "Error variant should have type 'error'");
    assert_eq!(json["value"], "something failed", "Error variant value mismatch");
}

#[test]
fn test_derived_result_rich_text_variant() {
    let spans = vec![
        StyledSpan {
            text: "bold".to_string(),
            bold: true,
            italic: false,
            underline: false,
            color: None,
        },
        StyledSpan::plain(" normal"),
    ];
    let result = DerivedResult::RichText(spans);
    let json = result.to_frontend_value();
    assert_eq!(json["type"], "richText", "RichText variant should have type 'richText'");
    assert!(json["value"].is_object(), "RichText value should be an object");
}

#[test]
fn test_styled_span_plain_constructor() {
    let span = StyledSpan::plain("hello");
    assert_eq!(span.text, "hello", "Plain span text mismatch");
    assert!(!span.bold, "Plain span should not be bold");
    assert!(!span.italic, "Plain span should not be italic");
    assert!(!span.underline, "Plain span should not be underlined");
    assert!(span.color.is_none(), "Plain span should have no color");
}

#[test]
fn test_derived_function_trait_name() {
    let func = TestFunction::new(DerivedResult::Text("x".to_string()));
    assert_eq!(func.name(), "test_function", "Function name mismatch");
}

#[test]
fn test_derived_function_trait_input_keys() {
    let func = TestFunction::new(DerivedResult::Text("x".to_string()));
    assert_eq!(func.input_keys(), vec!["input"], "Input keys mismatch");
}

#[test]
fn test_derived_function_trait_compute() {
    let func = TestFunction::new(DerivedResult::Number(99.0));
    let inputs: RowData = HashMap::new();
    let context = LookupContext::new();
    let result = func.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Number(99.0), "Compute result mismatch");
}

#[test]
fn test_derived_function_is_async_default() {
    let func = TestFunction::new(DerivedResult::Text("x".to_string()));
    assert!(!func.is_async(), "Default is_async should be false");
}

#[test]
fn test_derived_function_is_async_override() {
    let func = AsyncTestFunction;
    assert!(func.is_async(), "AsyncTestFunction should return true for is_async");
}

#[test]
fn test_registry_register_and_lookup() {
    let registry = FunctionRegistry::new();
    registry.register(Box::new(TestFunction::new(DerivedResult::Text("x".to_string()))));

    assert!(registry.contains("test_function"), "Registry should contain test_function");
    assert!(!registry.contains("nonexistent"), "Registry should not contain nonexistent");
}

#[test]
fn test_registry_with_function() {
    let registry = FunctionRegistry::new();
    registry.register(Box::new(TestFunction::new(DerivedResult::Text("hello".to_string()))));

    let result = registry.with_function("test_function", |f| {
        let inputs: RowData = HashMap::new();
        let context = LookupContext::new();
        f.compute(&inputs, &context)
    });
    assert_eq!(
        result,
        Some(DerivedResult::Text("hello".to_string())),
        "with_function should return computed result"
    );
}

#[test]
fn test_registry_lookup_missing_function() {
    let registry = FunctionRegistry::new();
    let result = registry.with_function("nonexistent", |_f| ());
    assert!(result.is_none(), "Looking up missing function should return None");
}

#[test]
#[should_panic(expected = "already registered")]
fn test_registry_duplicate_panics() {
    let registry = FunctionRegistry::new();
    registry.register(Box::new(TestFunction::new(DerivedResult::Text("a".to_string()))));
    registry.register(Box::new(TestFunction::new(DerivedResult::Text("b".to_string()))));
}

#[test]
fn test_registry_list_functions() {
    let registry = FunctionRegistry::new();
    registry.register(Box::new(TestFunction::new(DerivedResult::Text("x".to_string()))));
    registry.register(Box::new(AsyncTestFunction));

    let mut names = registry.list_functions();
    names.sort();
    assert_eq!(names, vec!["async_test", "test_function"], "Function list mismatch");
}

#[test]
fn test_lookup_context_new_is_empty() {
    let context = LookupContext::new();
    assert!(context.lookup_by_id("cards", "any-id").is_none(), "New context should have no data");
}

#[test]
fn test_lookup_context_add_and_lookup() {
    let mut context = LookupContext::new();
    let mut cards: HashMap<String, RowData> = HashMap::new();
    let mut row: RowData = HashMap::new();
    row.insert("name".to_string(), serde_json::json!("Test Card"));
    cards.insert("card-1".to_string(), row);
    context.add_table("cards", cards);

    let found = context.lookup_by_id("cards", "card-1");
    assert!(found.is_some(), "Should find card by ID");
    assert_eq!(
        found.unwrap().get("name"),
        Some(&serde_json::json!("Test Card")),
        "Card name mismatch"
    );
}

#[test]
fn test_lookup_context_any_table() {
    let mut context = LookupContext::new();
    let mut cards: HashMap<String, RowData> = HashMap::new();
    let mut row: RowData = HashMap::new();
    row.insert("name".to_string(), serde_json::json!("Found It"));
    cards.insert("abc-123".to_string(), row);
    context.add_table("cards", cards);

    let result = context.lookup_by_id_any_table("abc-123");
    assert!(result.is_some(), "Should find across all tables");
    let (table_name, data) = result.unwrap();
    assert_eq!(table_name, "cards", "Table name mismatch");
    assert_eq!(data.get("name"), Some(&serde_json::json!("Found It")), "Row data mismatch");
}

#[test]
fn test_lookup_context_any_table_miss() {
    let context = LookupContext::new();
    assert!(
        context.lookup_by_id_any_table("missing").is_none(),
        "Should return None for missing ID"
    );
}

#[test]
fn test_lookup_context_default() {
    let context = LookupContext::default();
    assert!(context.lookup_by_id("any", "any").is_none(), "Default context should be empty");
}
