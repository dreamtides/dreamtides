use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::derived::rich_text_converter;

/// The result of a derived column computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DerivedResult {
    /// Plain text result
    Text(String),
    /// Numeric result
    Number(f64),
    /// Boolean result
    Boolean(bool),
    /// Image URL or path result
    Image(String),
    /// Rich text with styled spans
    RichText(Vec<StyledSpan>),
    /// Error with message
    Error(String),
}

impl DerivedResult {
    /// Converts this result to a JSON value suitable for the frontend.
    pub fn to_frontend_value(&self) -> serde_json::Value {
        match self {
            DerivedResult::Text(s) => serde_json::json!({
                "type": "text",
                "value": s
            }),
            DerivedResult::Number(n) => serde_json::json!({
                "type": "number",
                "value": n
            }),
            DerivedResult::Boolean(b) => serde_json::json!({
                "type": "boolean",
                "value": b
            }),
            DerivedResult::Image(url) => serde_json::json!({
                "type": "image",
                "value": url
            }),
            DerivedResult::RichText(spans) => {
                let rich_text = rich_text_converter::styled_spans_to_univer_rich_text(spans);
                serde_json::json!({
                    "type": "richText",
                    "value": rich_text
                })
            }
            DerivedResult::Error(msg) => serde_json::json!({
                "type": "error",
                "value": msg
            }),
        }
    }
}

/// A styled span for rich text output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyledSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
}

impl StyledSpan {
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), bold: false, italic: false, underline: false, color: None }
    }
}

/// Row data containing field values keyed by column name.
pub type RowData = HashMap<String, serde_json::Value>;

/// Trait for derived column functions.
///
/// Derived functions compute values from row data that are displayed in
/// derived columns. The computation may be synchronous or asynchronous.
pub trait DerivedFunction: Send + Sync {
    /// Returns the unique string identifier for this function.
    fn name(&self) -> &'static str;

    /// Returns the list of TOML keys that this function reads from the row.
    fn input_keys(&self) -> Vec<&'static str>;

    /// Computes the derived value from the given row data.
    fn compute(&self, inputs: &RowData, context: &LookupContext) -> DerivedResult;

    /// Returns whether this function should run on the async thread pool.
    /// Default is false (synchronous execution).
    fn is_async(&self) -> bool {
        false
    }
}

/// Context for cross-table lookups.
///
/// Provides read access to data from other tables for functions like
/// CardLookup that need to resolve references.
pub struct LookupContext {
    /// Map of table name to table data, where each entry is a map of
    /// row UUID -> row data.
    tables: HashMap<String, HashMap<String, RowData>>,
}

impl LookupContext {
    /// Creates a new empty lookup context.
    pub fn new() -> Self {
        Self { tables: HashMap::new() }
    }

    /// Adds a table to the lookup context.
    ///
    /// The table data should be indexed by the "id" field of each row.
    pub fn add_table(&mut self, table_name: impl Into<String>, rows: HashMap<String, RowData>) {
        self.tables.insert(table_name.into(), rows);
    }

    /// Looks up a row by its ID in the specified table.
    pub fn lookup_by_id(&self, table_name: &str, id: &str) -> Option<&RowData> {
        self.tables.get(table_name).and_then(|table| table.get(id))
    }

    /// Looks up a row by its ID across all tables.
    ///
    /// Searches all tables for a row with the given ID. Returns the first
    /// match found, along with the table name.
    pub fn lookup_by_id_any_table(&self, id: &str) -> Option<(&str, &RowData)> {
        for (table_name, table) in &self.tables {
            if let Some(row) = table.get(id) {
                return Some((table_name.as_str(), row));
            }
        }
        None
    }
}

impl Default for LookupContext {
    fn default() -> Self {
        Self::new()
    }
}
