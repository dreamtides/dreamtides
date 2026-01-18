use serde::Serialize;

use crate::cli::output_format;
use crate::cli::shared_options::ListFormat;
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::index::document_types::DocumentRow;

/// JSON output format for a document in `lat list`.
#[derive(Debug, Clone, Serialize)]
pub struct ListDocumentJson {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    pub is_closed: bool,
    pub labels: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Outputs documents in text format based on the selected format.
pub fn output_text(documents: &[DocumentRow], format: ListFormat) {
    if documents.is_empty() {
        println!("No documents found.");
        return;
    }

    for doc in documents {
        let line = match format {
            ListFormat::Rich => format_rich(doc),
            ListFormat::Compact => format_compact(doc),
            ListFormat::Oneline => format_rich(doc),
        };
        println!("{line}");
    }
}

/// Outputs documents in JSON format.
pub fn output_json(
    documents: &[DocumentRow],
    labels_list: &[Vec<String>],
) -> Result<(), LatticeError> {
    let json_docs: Vec<ListDocumentJson> = documents
        .iter()
        .enumerate()
        .map(|(i, doc)| ListDocumentJson {
            id: doc.id.clone(),
            name: doc.name.clone(),
            description: doc.description.clone(),
            path: doc.path.clone(),
            task_type: doc.task_type.map(format_task_type),
            priority: doc.priority,
            is_closed: doc.is_closed,
            labels: labels_list.get(i).cloned().unwrap_or_default(),
            created_at: doc.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: doc.updated_at.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    let json_str = output_format::output_json_array(&json_docs).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}

/// Formats a document in rich format: `LXXXXX [bug/P1] name - description`
///
/// For closed tasks: `LXXXXX [task/P2/closed] name - description`
/// For knowledge base: `LXXXXX [doc] name - description`
fn format_rich(doc: &DocumentRow) -> String {
    let type_indicator = format_type_indicator(doc);
    format!("{} [{}] {} - {}", doc.id, type_indicator, doc.name, doc.description)
}

/// Formats a document in compact format: `LXXXXX  name`
fn format_compact(doc: &DocumentRow) -> String {
    format!("{}  {}", doc.id, doc.name)
}

/// Formats the type indicator for display.
///
/// - Tasks: `bug/P1` or `task/P2/closed`
/// - Knowledge base: `doc`
fn format_type_indicator(doc: &DocumentRow) -> String {
    match doc.task_type {
        Some(task_type) => {
            let type_str = format_task_type(task_type);
            let priority_str = format_priority(doc.priority);
            if doc.is_closed {
                format!("{type_str}/{priority_str}/closed")
            } else {
                format!("{type_str}/{priority_str}")
            }
        }
        None => "doc".to_string(),
    }
}

fn format_priority(priority: Option<u8>) -> String {
    match priority {
        Some(p) => format!("P{p}"),
        None => "P2".to_string(),
    }
}

fn format_task_type(task_type: TaskType) -> String {
    match task_type {
        TaskType::Bug => "bug".to_string(),
        TaskType::Feature => "feature".to_string(),
        TaskType::Task => "task".to_string(),
        TaskType::Chore => "chore".to_string(),
    }
}
