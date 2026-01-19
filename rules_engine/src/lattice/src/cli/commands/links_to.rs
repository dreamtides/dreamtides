use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::link_display;
use crate::cli::commands::link_display::LinkDocumentInfo;
use crate::cli::structure_args::LinksToArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::link_queries::LinkRow;
use crate::index::{document_queries, link_queries};

/// Executes the `lat links-to` command.
///
/// Shows documents that link to the given document (incoming links/backlinks).
/// Validates that the provided ID exists before querying links.
#[instrument(skip_all, name = "links_to_command", fields(id = %args.id))]
pub fn execute(context: CommandContext, args: LinksToArgs) -> LatticeResult<()> {
    info!(id = %args.id, "Executing links-to command");

    let target_doc = document_queries::lookup_by_id(&context.conn, &args.id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id.clone() })?;
    debug!(id = %args.id, path = %target_doc.path, "Target document found");

    let incoming_links = link_queries::query_incoming(&context.conn, &args.id)?;
    info!(count = incoming_links.len(), "Found incoming links");

    let linking_documents = resolve_linking_documents(&context, &incoming_links)?;

    if context.global.json {
        output_json(&linking_documents)?;
    } else {
        output_text(&linking_documents, &target_doc.name);
    }

    Ok(())
}

/// Resolves link rows to document information for incoming links.
///
/// Looks up each linking document by its source ID. Links from non-existent
/// documents are logged as warnings and skipped (this can happen if the
/// index is stale or the source was deleted).
fn resolve_linking_documents(
    context: &CommandContext,
    links: &[LinkRow],
) -> LatticeResult<Vec<LinkDocumentInfo>> {
    let mut results = Vec::with_capacity(links.len());

    for link in links {
        match document_queries::lookup_by_id(&context.conn, &link.source_id)? {
            Some(doc) => {
                results.push(LinkDocumentInfo {
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
                    source_id = %link.source_id,
                    "Linking document not found in index, skipping"
                );
            }
        }
    }

    Ok(results)
}

/// Outputs linking documents in text format.
fn output_text(documents: &[LinkDocumentInfo], target_name: &str) {
    if documents.is_empty() {
        println!("No incoming links to {}.", target_name);
        return;
    }

    let count_str =
        output_format::format_count(documents.len(), "document links", "documents link");
    println!("{count_str} to {}:", color_theme::bold(target_name));
    println!();

    for doc in documents {
        let id_str = color_theme::lattice_id(&doc.id);
        let type_str = link_display::format_type_priority(doc);
        let name_str = color_theme::bold(&doc.name);
        let status_str =
            if doc.is_closed { color_theme::muted("[closed]").to_string() } else { String::new() };
        let link_type_str = color_theme::muted(format!("({})", doc.link_type));

        println!("  {id_str} {type_str} {name_str} {status_str} {link_type_str}");
        println!("    {}", doc.description);
        println!("    {}", color_theme::path(&doc.path));
    }
}

/// Outputs linking documents in JSON format.
fn output_json(documents: &[LinkDocumentInfo]) -> LatticeResult<()> {
    let json_str = output_format::output_json_array(documents).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
