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
