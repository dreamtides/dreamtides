//! Shared display types and formatting for link-related commands.
//!
//! This module provides common functionality for `lat links-from` and
//! `lat links-to` commands.

use serde::Serialize;

use crate::cli::color_theme;

/// Information about a linked document for display and JSON output.
///
/// Used by both `lat links-from` (outgoing links) and `lat links-to`
/// (incoming links) commands.
#[derive(Debug, Clone, Serialize)]
pub struct LinkDocumentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    pub is_closed: bool,
}

/// Formats the type and priority indicator for a document.
pub fn format_type_priority(doc: &LinkDocumentInfo) -> String {
    match (&doc.task_type, doc.priority) {
        (Some(task_type), Some(priority)) => {
            color_theme::task_type(format!("[{}/P{}]", task_type, priority)).to_string()
        }
        (Some(task_type), None) => color_theme::task_type(format!("[{}]", task_type)).to_string(),
        (None, _) => color_theme::muted("[doc]").to_string(),
    }
}
