use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::LinksFromArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::link_queries::LinkRow;
use crate::index::{document_queries, link_queries};

/// Executes the `lat links-from` command.
///
/// Shows documents that the given document links to (outgoing links).
/// Validates that the provided ID exists before querying links.
#[instrument(skip_all, name = "links_from_command", fields(id = %args.id))]
pub fn execute(context: CommandContext, args: LinksFromArgs) -> LatticeResult<()> {
    info!(id = %args.id, "Executing links-from command");

    let source_doc = document_queries::lookup_by_id(&context.conn, &args.id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id.clone() })?;
    debug!(id = %args.id, path = %source_doc.path, "Source document found");

    let outgoing_links = link_queries::query_outgoing(&context.conn, &args.id)?;
    info!(count = outgoing_links.len(), "Found outgoing links");

    let linked_documents = resolve_linked_documents(&context, &outgoing_links)?;

    if context.global.json {
        output_json(&linked_documents)?;
    } else {
        output_text(&linked_documents, &source_doc.name);
    }

    Ok(())
}

/// Information about a linked document for display and JSON output.
#[derive(Debug, Clone, Serialize)]
struct LinkedDocumentInfo {
    id: String,
    name: String,
    description: String,
    path: String,
    link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    is_closed: bool,
}

/// Resolves link rows to document information.
///
/// Looks up each linked document by its target ID. Links to non-existent
/// documents are logged as warnings and skipped (this can happen if the
/// index is stale or the target was deleted).
fn resolve_linked_documents(
    context: &CommandContext,
    links: &[LinkRow],
) -> LatticeResult<Vec<LinkedDocumentInfo>> {
    let mut results = Vec::with_capacity(links.len());

    for link in links {
        match document_queries::lookup_by_id(&context.conn, &link.target_id)? {
            Some(doc) => {
                results.push(LinkedDocumentInfo {
                    id: doc.id,
                    name: doc.name,
                    description: doc.description,
                    path: doc.path,
                    link_type: link.link_type.to_string(),
                    task_type: doc.task_type.map(|t| t.to_string()),
                    priority: doc.priority,
                    is_closed: doc.is_closed,
                });
            }
            None => {
                debug!(
                    target_id = %link.target_id,
                    "Linked document not found in index, skipping"
                );
            }
        }
    }

    Ok(results)
}

/// Outputs linked documents in text format.
fn output_text(documents: &[LinkedDocumentInfo], source_name: &str) {
    if documents.is_empty() {
        println!("No outgoing links from {}.", source_name);
        return;
    }

    let count_str =
        output_format::format_count(documents.len(), "linked document", "linked documents");
    println!("{count_str} from {}:", color_theme::bold(source_name));
    println!();

    for doc in documents {
        let id_str = color_theme::lattice_id(&doc.id);
        let type_str = format_type_priority(doc);
        let name_str = color_theme::bold(&doc.name);
        let status_str =
            if doc.is_closed { color_theme::muted("[closed]").to_string() } else { String::new() };
        let link_type_str = color_theme::muted(format!("({})", doc.link_type));

        println!("  {id_str} {type_str} {name_str} {status_str} {link_type_str}");
        println!("    {}", doc.description);
        println!("    {}", color_theme::path(&doc.path));
    }
}

/// Formats the type and priority indicator for a document.
fn format_type_priority(doc: &LinkedDocumentInfo) -> String {
    match (&doc.task_type, doc.priority) {
        (Some(task_type), Some(priority)) => {
            color_theme::task_type(format!("[{}/P{}]", task_type, priority)).to_string()
        }
        (Some(task_type), None) => color_theme::task_type(format!("[{}]", task_type)).to_string(),
        (None, _) => color_theme::muted("[doc]").to_string(),
    }
}

/// Outputs linked documents in JSON format.
fn output_json(documents: &[LinkedDocumentInfo]) -> LatticeResult<()> {
    let json_str = output_format::output_json_array(documents).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
