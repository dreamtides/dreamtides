use serde::{Deserialize, Serialize};

/// Represents a single cell update request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellUpdate {
    pub row_index: usize,
    pub column_key: String,
    pub value: serde_json::Value,
}

/// Result of a table save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTableResult {
    pub uuids_generated: bool,
}

/// Result of a cell save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCellResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_values: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Result of a batch save operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBatchResult {
    pub success: bool,
    pub applied_count: usize,
    pub failed_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failed_updates: Vec<FailedUpdate>,
}

/// Information about a failed cell update within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedUpdate {
    pub row_index: usize,
    pub column_key: String,
    pub reason: String,
}

/// Result of a row add operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRowResult {
    pub success: bool,
    pub row_index: usize,
}

/// Result of a row delete operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRowResult {
    pub success: bool,
    pub deleted_index: usize,
}
