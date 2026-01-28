use std::collections::HashMap;

use tv_lib::derived::card_lookup::CardLookupFunction;
use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};

use crate::test_utils::harness::TvTestHarness;

/// Helper to create a LookupContext from a loaded TOML table.
fn create_context_from_toml(harness: &TvTestHarness, path: &std::path::Path) -> LookupContext {
    let table = harness.load_table(path, "cards").expect("Should load cards table");
    let mut context = LookupContext::new();

    // Index rows by their "id" field
    let id_idx = table.headers.iter().position(|h| h == "id").expect("Table should have id column");

    let mut cards: HashMap<String, RowData> = HashMap::new();
    for row in &table.rows {
        if let Some(serde_json::Value::String(id)) = row.get(id_idx) {
            let mut row_data: RowData = HashMap::new();
            for (i, header) in table.headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    row_data.insert(header.clone(), value.clone());
                }
            }
            cards.insert(id.clone(), row_data);
        }
    }

    context.add_table("cards", cards);
    context
}

#[test]
fn test_card_lookup_with_real_toml() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "uuid-001"
name = "Dragon Knight"
cost = 5

[[cards]]
id = "uuid-002"
name = "Forest Spirit"
cost = 3

[[cards]]
id = "uuid-003"
name = "Thunder Mage"
cost = 4
"#,
    );

    let context = create_context_from_toml(&harness, &path);
    let function = CardLookupFunction::new();

    // Look up an existing card
    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("uuid-002"));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text("Forest Spirit".to_string()));
}

#[test]
fn test_card_lookup_unknown_uuid() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "uuid-001"
name = "Dragon Knight"
"#,
    );

    let context = create_context_from_toml(&harness, &path);
    let function = CardLookupFunction::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("nonexistent-uuid"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("Unknown Card"), "Should report unknown card: {msg}");
        }
        _ => panic!("Expected error result for unknown UUID"),
    }
}

#[test]
fn test_card_lookup_multiple_lookups_use_cache() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "cached-card-uuid"
name = "Cached Card"
"#,
    );

    let context = create_context_from_toml(&harness, &path);
    let function = CardLookupFunction::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("cached-card-uuid"));

    // First lookup
    let result1 = function.compute(&inputs, &context);
    assert_eq!(result1, DerivedResult::Text("Cached Card".to_string()));

    // Second lookup (should use cache)
    let result2 = function.compute(&inputs, &context);
    assert_eq!(result2, DerivedResult::Text("Cached Card".to_string()));
}

#[test]
fn test_card_lookup_with_unicode_names() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "jp-card"
name = "竜騎士"

[[cards]]
id = "emoji-card"
name = "Magic Card ✨"
"#,
    );

    let context = create_context_from_toml(&harness, &path);
    let function = CardLookupFunction::new();

    // Japanese name
    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("jp-card"));
    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text("竜騎士".to_string()));

    // Emoji name
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("emoji-card"));
    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text("Magic Card ✨".to_string()));
}

#[test]
fn test_card_lookup_empty_table() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "empty_cards.toml",
        r#"
[[cards]]
id = "only-one"
name = "Lonely Card"
"#,
    );

    // Create empty context (no cards added)
    let context = LookupContext::new();
    let function = CardLookupFunction::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("only-one"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("Unknown Card"), "Should report unknown card: {msg}");
        }
        _ => panic!("Expected error when context has no tables"),
    }
    // Suppress unused variable warning
    drop(path);
}

#[test]
fn test_card_lookup_card_without_name_field() {
    let function = CardLookupFunction::new();

    // Create context with a card that has no name field
    let mut context = LookupContext::new();
    let mut cards: HashMap<String, RowData> = HashMap::new();

    let mut card: RowData = HashMap::new();
    card.insert("id".to_string(), serde_json::json!("nameless-card"));
    card.insert("cost".to_string(), serde_json::json!(3));
    // No "name" field
    cards.insert("nameless-card".to_string(), card);

    context.add_table("cards", cards);

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("nameless-card"));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("missing 'name' field"), "Should report missing name: {msg}");
        }
        _ => panic!("Expected error for card without name field"),
    }
}

#[test]
fn test_card_lookup_in_effects_table_referencing_cards() {
    let harness = TvTestHarness::new();

    // Create a cards table
    let cards_path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "card-uuid-123"
name = "Fire Elemental"
cost = 4
"#,
    );

    // Create an effects table that references cards
    let _effects_path = harness.create_toml_file(
        "effects.toml",
        r#"
[[effects]]
id = "effect-001"
target_card_id = "card-uuid-123"
damage = 3
"#,
    );

    // Load cards into context
    let context = create_context_from_toml(&harness, &cards_path);
    let function = CardLookupFunction::new();

    // Simulate looking up from effects table
    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("card-uuid-123"));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text("Fire Elemental".to_string()));
}

#[test]
fn test_derived_function_trait_implementation() {
    let function = CardLookupFunction::new();

    assert_eq!(function.name(), "card_lookup");
    assert_eq!(function.input_keys(), vec!["card_id"]);
    assert!(!function.is_async());
}

#[test]
fn test_card_lookup_empty_reference() {
    let function = CardLookupFunction::new();
    let context = LookupContext::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!(""));

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_card_lookup_null_reference() {
    let function = CardLookupFunction::new();
    let context = LookupContext::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::Value::Null);

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_card_lookup_missing_reference_field() {
    let function = CardLookupFunction::new();
    let context = LookupContext::new();

    let inputs: RowData = HashMap::new();

    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text(String::new()));
}

#[test]
fn test_card_lookup_invalid_type() {
    let function = CardLookupFunction::new();
    let context = LookupContext::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!(12345));

    let result = function.compute(&inputs, &context);
    match result {
        DerivedResult::Error(msg) => {
            assert!(
                msg.contains("Invalid card reference type"),
                "Error should mention invalid type: {msg}"
            );
        }
        _ => panic!("Expected error result for invalid type, got: {result:?}"),
    }
}

#[test]
fn test_card_lookup_default_constructor() {
    let function = CardLookupFunction::default();

    assert_eq!(function.name(), "card_lookup");
    assert_eq!(function.input_keys(), vec!["card_id"]);
    assert!(!function.is_async());
}

#[test]
fn test_clear_cache() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cards.toml",
        r#"
[[cards]]
id = "cache-test-card"
name = "Cache Test"
"#,
    );

    let context = create_context_from_toml(&harness, &path);
    let function = CardLookupFunction::new();

    let mut inputs: RowData = HashMap::new();
    inputs.insert("referenced_card_id".to_string(), serde_json::json!("cache-test-card"));

    // Populate cache
    function.compute(&inputs, &context);

    // Clear cache
    function.clear_cache();

    // Should still work after clearing
    let result = function.compute(&inputs, &context);
    assert_eq!(result, DerivedResult::Text("Cache Test".to_string()));
}
