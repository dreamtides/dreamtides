use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::ChildrenArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, SortColumn, SortOrder};
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;

/// Executes the `lat children` command.
///
/// Lists documents under a root document's directory. Validates that the
/// provided ID refers to a root document (one whose filename matches its
/// containing directory name, e.g., `api/api.md`).
#[instrument(skip_all, name = "children_command", fields(root_id = %args.root_id))]
pub fn execute(context: CommandContext, args: ChildrenArgs) -> LatticeResult<()> {
    info!(root_id = %args.root_id, recursive = args.recursive, tasks = args.tasks, docs = args.docs, "Executing children command");

    if args.tasks && args.docs {
        return Err(LatticeError::ConflictingOptions {
            option1: "--tasks".to_string(),
            option2: "--docs".to_string(),
        });
    }

    let root_doc = lookup_root_document(&context, &args.root_id)?;
    let directory_path = extract_directory_path(&root_doc.path);
    debug!(directory_path = %directory_path, "Extracted directory path from root document");

    let children = fetch_children(&context, &directory_path, &args)?;
    info!(count = children.len(), "Found child documents");

    if context.global.json {
        output_json(&children)?;
    } else {
        output_text(&children, &root_doc.name);
    }

    Ok(())
}

/// Information about a child document for display and JSON output.
#[derive(Debug, Clone, Serialize)]
struct ChildDocumentInfo {
    id: String,
    name: String,
    description: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    is_closed: bool,
    in_tasks_dir: bool,
    in_docs_dir: bool,
}

/// Looks up a document by ID and verifies it is a root document.
fn lookup_root_document(context: &CommandContext, root_id: &str) -> LatticeResult<DocumentRow> {
    let doc = document_queries::lookup_by_id(&context.conn, root_id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: root_id.to_string() })?;

    if !doc.is_root {
        return Err(LatticeError::InvalidArgument {
            message: format!(
                "Document {} ({}) is not a root document. Root documents have filenames \
                 matching their containing directory (e.g., api/api.md).",
                root_id, doc.name
            ),
        });
    }

    debug!(id = %root_id, path = %doc.path, "Verified root document");
    Ok(doc)
}

/// Extracts the directory path from a root document's file path.
///
/// For a root document like `api/api.md`, this returns `api`.
fn extract_directory_path(file_path: &str) -> String {
    file_path
        .rsplit_once('/')
        .map(|(dir, _filename)| dir.to_string())
        .unwrap_or_else(|| ".".to_string())
}

/// Fetches child documents under the root's directory.
fn fetch_children(
    context: &CommandContext,
    directory_path: &str,
    args: &ChildrenArgs,
) -> LatticeResult<Vec<ChildDocumentInfo>> {
    let path_prefix = format!("{directory_path}/");

    let mut filter = DocumentFilter::including_closed()
        .with_path_prefix(path_prefix)
        .with_is_root(false)
        .sort_by(SortColumn::Path)
        .sort_order(SortOrder::Ascending);

    if args.tasks {
        filter = filter.with_in_tasks_dir(true);
    } else if args.docs {
        filter = filter.with_in_docs_dir(true);
    }

    let documents = document_queries::query(&context.conn, &filter)?;
    debug!(count = documents.len(), "Retrieved documents from index");

    let children: Vec<ChildDocumentInfo> = documents
        .into_iter()
        .filter(|doc| args.recursive || is_direct_child(directory_path, &doc.path))
        .map(|doc| ChildDocumentInfo {
            id: doc.id,
            name: doc.name,
            description: doc.description,
            path: doc.path,
            task_type: doc.task_type.map(|t| t.to_string()),
            priority: doc.priority,
            is_closed: doc.is_closed,
            in_tasks_dir: doc.in_tasks_dir,
            in_docs_dir: doc.in_docs_dir,
        })
        .collect();

    Ok(children)
}

/// Checks if a document is a direct child of the given directory.
///
/// A document is a direct child if it is in `{directory}/docs/` or
/// `{directory}/tasks/` with no additional nesting.
fn is_direct_child(directory_path: &str, doc_path: &str) -> bool {
    let Some(relative) = doc_path.strip_prefix(directory_path) else {
        return false;
    };

    let relative = relative.strip_prefix('/').unwrap_or(relative);

    let parts: Vec<&str> = relative.split('/').collect();
    match parts.as_slice() {
        [subdir, _filename] if *subdir == "docs" || *subdir == "tasks" => true,
        [subdir, ".closed", _filename] if *subdir == "tasks" => true,
        _ => false,
    }
}

/// Outputs child documents in text format.
fn output_text(children: &[ChildDocumentInfo], root_name: &str) {
    if children.is_empty() {
        println!("No child documents found under {}.", root_name);
        return;
    }

    let count_str =
        output_format::format_count(children.len(), "child document", "child documents");
    println!("{count_str} under {}:", color_theme::bold(root_name));
    println!();

    for child in children {
        let id_str = color_theme::lattice_id(&child.id);
        let type_str = format_type_priority(child);
        let name_str = color_theme::bold(&child.name);
        let status_str = if child.is_closed {
            color_theme::muted("[closed]").to_string()
        } else {
            String::new()
        };

        println!("  {id_str} {type_str} {name_str} {status_str}");
        println!("    {}", child.description);
        println!("    {}", color_theme::path(&child.path));
    }
}

/// Formats the type and priority indicator for a document.
fn format_type_priority(child: &ChildDocumentInfo) -> String {
    match (&child.task_type, child.priority) {
        (Some(task_type), Some(priority)) => {
            color_theme::task_type(format!("[{}/P{}]", task_type, priority)).to_string()
        }
        (Some(task_type), None) => color_theme::task_type(format!("[{}]", task_type)).to_string(),
        (None, _) if child.in_docs_dir => color_theme::muted("[doc]").to_string(),
        (None, _) => String::new(),
    }
}

/// Outputs child documents in JSON format.
fn output_json(children: &[ChildDocumentInfo]) -> LatticeResult<()> {
    let json_str = output_format::output_json_array(children).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
