use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::OrphansArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;

/// Executes the `lat orphans` command.
///
/// Finds documents with no incoming links (orphans). Root documents naturally
/// have fewer incoming links since they serve as hierarchy anchors, so they
/// can be excluded with --exclude-roots.
#[instrument(skip_all, name = "orphans_command")]
pub fn execute(context: CommandContext, args: OrphansArgs) -> LatticeResult<()> {
    info!(exclude_roots = args.exclude_roots, path = ?args.path, "Executing orphans command");

    let orphans = query_orphan_documents(&context, &args)?;
    info!(count = orphans.len(), "Found orphan documents");

    if context.global.json {
        output_json(&orphans)?;
    } else {
        output_text(&orphans);
    }

    Ok(())
}

/// JSON output format for orphan documents.
#[derive(Debug, Clone, Serialize)]
struct OrphanDocumentJson {
    id: String,
    name: String,
    description: String,
    path: String,
    is_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
}

/// Queries for documents with no incoming links.
fn query_orphan_documents(
    context: &CommandContext,
    args: &OrphansArgs,
) -> LatticeResult<Vec<DocumentRow>> {
    debug!("Querying for orphan documents");

    let filter = build_filter(args);
    let candidates = document_queries::query(&context.conn, &filter)?;

    debug!(candidates = candidates.len(), "Evaluating candidate documents for orphan status");

    let orphans: Vec<DocumentRow> = candidates
        .into_iter()
        .filter(|doc| doc.backlink_count == 0)
        .filter(|doc| !args.exclude_roots || !doc.is_root)
        .collect();

    Ok(orphans)
}

/// Builds a document filter from CLI arguments.
fn build_filter(args: &OrphansArgs) -> DocumentFilter {
    DocumentFilter { include_closed: false, path_prefix: args.path.clone(), ..Default::default() }
}

/// Outputs orphan documents in text format.
fn output_text(documents: &[DocumentRow]) {
    if documents.is_empty() {
        println!("No orphan documents found.");
        return;
    }

    let count_str =
        output_format::format_count(documents.len(), "orphan document", "orphan documents");
    println!("{count_str}:");
    println!();

    for doc in documents {
        let id_str = color_theme::lattice_id(&doc.id);
        let type_str = format_type_priority(doc);
        let name_str = color_theme::bold(&doc.name);
        let root_str =
            if doc.is_root { color_theme::muted("[root]").to_string() } else { String::new() };

        println!("  {id_str} {type_str} {name_str} {root_str}");
        println!("    {}", doc.description);
        println!("    {}", color_theme::path(&doc.path));
    }
}

/// Formats task type and priority for display.
fn format_type_priority(doc: &DocumentRow) -> String {
    match (doc.task_type, doc.priority) {
        (Some(task_type), Some(priority)) => {
            let type_str = task_type.to_string();
            let priority_str = output_format::format_priority(priority);
            color_theme::muted(format!("[{type_str}/{priority_str}]")).to_string()
        }
        (Some(task_type), None) => color_theme::muted(format!("[{}]", task_type)).to_string(),
        (None, _) => color_theme::muted("[doc]").to_string(),
    }
}

/// Outputs orphan documents in JSON format.
fn output_json(documents: &[DocumentRow]) -> LatticeResult<()> {
    let json_docs: Vec<OrphanDocumentJson> = documents
        .iter()
        .map(|doc| OrphanDocumentJson {
            id: doc.id.clone(),
            name: doc.name.clone(),
            description: doc.description.clone(),
            path: doc.path.clone(),
            is_root: doc.is_root,
            task_type: doc.task_type.map(|t| t.to_string()),
            priority: doc.priority,
        })
        .collect();

    let json_str = output_format::output_json_array(&json_docs).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
