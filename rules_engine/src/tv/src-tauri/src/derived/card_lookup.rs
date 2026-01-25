use std::collections::HashMap;
use std::sync::RwLock;

use crate::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};

/// A derived function that looks up card names from UUID references.
///
/// Given a cell value containing a card UUID, this function searches loaded
/// tables for a matching "id" field and returns the "name" field from that row.
pub struct CardLookupFunction {
    /// Cache of UUID -> card name mappings for performance.
    cache: RwLock<HashMap<String, String>>,
}

impl CardLookupFunction {
    pub fn new() -> Self {
        Self { cache: RwLock::new(HashMap::new()) }
    }

    /// Clears the lookup cache.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    fn lookup_card_name(&self, uuid: &str, context: &LookupContext) -> DerivedResult {
        // Check cache first
        if let Ok(cache) = self.cache.read() {
            if let Some(name) = cache.get(uuid) {
                return DerivedResult::Text(name.clone());
            }
        }

        // Search all tables for the card
        match context.lookup_by_id_any_table(uuid) {
            Some((_table_name, row_data)) => {
                // Extract the name field
                let name = match row_data.get("name") {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => {
                        return DerivedResult::Error(format!(
                            "Card found but missing 'name' field: {uuid}"
                        ));
                    }
                };

                // Cache the result
                if let Ok(mut cache) = self.cache.write() {
                    cache.insert(uuid.to_string(), name.clone());
                }

                DerivedResult::Text(name)
            }
            None => DerivedResult::Error(format!("Unknown Card: {uuid}")),
        }
    }
}

impl Default for CardLookupFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DerivedFunction for CardLookupFunction {
    fn name(&self) -> &'static str {
        "card_lookup"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec!["referenced_card_id"]
    }

    fn compute(&self, inputs: &RowData, context: &LookupContext) -> DerivedResult {
        // Get the referenced card ID from inputs
        let uuid = match inputs.get("referenced_card_id") {
            Some(serde_json::Value::String(s)) => s.as_str(),
            Some(serde_json::Value::Null) | None => {
                return DerivedResult::Text(String::new());
            }
            Some(other) => {
                return DerivedResult::Error(format!(
                    "Invalid card reference type: expected string, got {}",
                    json_type_name(other)
                ));
            }
        };

        if uuid.is_empty() {
            return DerivedResult::Text(String::new());
        }

        self.lookup_card_name(uuid, context)
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
    use super::*;

    fn create_test_context() -> LookupContext {
        let mut context = LookupContext::new();

        // Add a cards table with test data
        let mut cards: HashMap<String, RowData> = HashMap::new();

        let mut card1: RowData = HashMap::new();
        card1.insert("id".to_string(), serde_json::json!("d31723e7-8faa-41e3-89b6-c30d8fdf9cca"));
        card1.insert("name".to_string(), serde_json::json!("Titan of Forgotten Echoes"));
        card1.insert("cost".to_string(), serde_json::json!(6));
        cards.insert("d31723e7-8faa-41e3-89b6-c30d8fdf9cca".to_string(), card1);

        let mut card2: RowData = HashMap::new();
        card2.insert("id".to_string(), serde_json::json!("6d7969d2-9d18-4a38-8c26-c900aca5f1d4"));
        card2.insert("name".to_string(), serde_json::json!("Beacon of Tomorrow"));
        card2.insert("cost".to_string(), serde_json::json!(2));
        cards.insert("6d7969d2-9d18-4a38-8c26-c900aca5f1d4".to_string(), card2);

        context.add_table("cards", cards);
        context
    }

    #[test]
    fn test_lookup_existing_card() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert(
            "referenced_card_id".to_string(),
            serde_json::json!("d31723e7-8faa-41e3-89b6-c30d8fdf9cca"),
        );

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text("Titan of Forgotten Echoes".to_string()));
    }

    #[test]
    fn test_lookup_unknown_card() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert(
            "referenced_card_id".to_string(),
            serde_json::json!("nonexistent-uuid-12345"),
        );

        let result = function.compute(&inputs, &context);
        match result {
            DerivedResult::Error(msg) => {
                assert!(msg.contains("Unknown Card"), "Error should mention unknown card: {msg}");
            }
            _ => panic!("Expected error result, got: {result:?}"),
        }
    }

    #[test]
    fn test_lookup_empty_reference() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("referenced_card_id".to_string(), serde_json::json!(""));

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_lookup_null_reference() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert("referenced_card_id".to_string(), serde_json::Value::Null);

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_lookup_missing_reference_field() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let inputs: RowData = HashMap::new();

        let result = function.compute(&inputs, &context);
        assert_eq!(result, DerivedResult::Text(String::new()));
    }

    #[test]
    fn test_lookup_invalid_type() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

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
            _ => panic!("Expected error result, got: {result:?}"),
        }
    }

    #[test]
    fn test_cache_populated_after_lookup() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert(
            "referenced_card_id".to_string(),
            serde_json::json!("6d7969d2-9d18-4a38-8c26-c900aca5f1d4"),
        );

        // First lookup
        let result1 = function.compute(&inputs, &context);
        assert_eq!(result1, DerivedResult::Text("Beacon of Tomorrow".to_string()));

        // Verify cache is populated
        let cache = function.cache.read().unwrap();
        assert_eq!(
            cache.get("6d7969d2-9d18-4a38-8c26-c900aca5f1d4"),
            Some(&"Beacon of Tomorrow".to_string())
        );
    }

    #[test]
    fn test_cache_clear() {
        let function = CardLookupFunction::new();
        let context = create_test_context();

        let mut inputs: RowData = HashMap::new();
        inputs.insert(
            "referenced_card_id".to_string(),
            serde_json::json!("6d7969d2-9d18-4a38-8c26-c900aca5f1d4"),
        );

        // Populate cache
        function.compute(&inputs, &context);
        assert!(!function.cache.read().unwrap().is_empty());

        // Clear cache
        function.clear_cache();
        assert!(function.cache.read().unwrap().is_empty());
    }

    #[test]
    fn test_function_name() {
        let function = CardLookupFunction::new();
        assert_eq!(function.name(), "card_lookup");
    }

    #[test]
    fn test_input_keys() {
        let function = CardLookupFunction::new();
        assert_eq!(function.input_keys(), vec!["referenced_card_id"]);
    }

    #[test]
    fn test_is_not_async() {
        let function = CardLookupFunction::new();
        assert!(!function.is_async());
    }
}
