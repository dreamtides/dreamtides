use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::RootsArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, SortColumn, SortOrder};
use crate::index::{directory_roots, document_queries};

/// Executes the `lat roots` command.
///
/// Lists all root documents (documents whose filename matches their containing
/// directory name, e.g., `api/api.md`) with the count of child documents under
/// each root's directory.
#[instrument(skip_all, name = "roots_command")]
pub fn execute(context: CommandContext, _args: RootsArgs) -> LatticeResult<()> {
    info!("Executing roots command");

    let roots = fetch_roots(&context)?;

    info!(count = roots.len(), "Found root documents");

    if context.global.json {
        output_json(&roots)?;
    } else {
        output_text(&roots);
    }

    Ok(())
}

/// Information about a root document for display and JSON output.
#[derive(Debug, Clone, Serialize)]
struct RootDocumentInfo {
    id: String,
    name: String,
    description: String,
    path: String,
    child_count: u64,
}

/// Fetches all root documents with their child counts.
fn fetch_roots(context: &CommandContext) -> LatticeResult<Vec<RootDocumentInfo>> {
    let directory_roots = directory_roots::list_all(&context.conn)?;
    debug!(count = directory_roots.len(), "Retrieved directory root entries");

    let mut roots = Vec::new();
    for dir_root in &directory_roots {
        let Some(doc) = document_queries::lookup_by_id(&context.conn, &dir_root.root_id)? else {
            debug!(id = dir_root.root_id, "Root document not found in index, skipping");
            continue;
        };

        let child_count = count_children(context, &dir_root.directory_path)?;

        roots.push(RootDocumentInfo {
            id: doc.id,
            name: doc.name,
            description: doc.description,
            path: doc.path,
            child_count,
        });
    }

    roots.sort_by(|a, b| a.path.cmp(&b.path));
    debug!(count = roots.len(), "Prepared root document list");

    Ok(roots)
}

/// Counts documents under a root's directory (excluding the root itself).
fn count_children(context: &CommandContext, directory_path: &str) -> LatticeResult<u64> {
    let filter = DocumentFilter::including_closed()
        .with_path_prefix(format!("{directory_path}/"))
        .with_is_root(false)
        .sort_by(SortColumn::Path)
        .sort_order(SortOrder::Ascending);

    document_queries::count(&context.conn, &filter)
}

/// Outputs root documents in text format.
fn output_text(roots: &[RootDocumentInfo]) {
    if roots.is_empty() {
        println!("No root documents found.");
        return;
    }

    let count_str = output_format::format_count(roots.len(), "root document", "root documents");
    println!("{count_str}:");
    println!();

    for root in roots {
        let id_str = color_theme::lattice_id(&root.id);
        let name_str = color_theme::bold(&root.name);
        let child_str = output_format::format_count(root.child_count as usize, "child", "children");
        let child_display = color_theme::muted(format!("({child_str})"));

        println!("  {id_str}  {name_str} {child_display}");
        println!("    {}", root.description);
        println!("    {}", color_theme::path(&root.path));
    }
}

/// Outputs root documents in JSON format.
fn output_json(roots: &[RootDocumentInfo]) -> LatticeResult<()> {
    let json_str = output_format::output_json_array(roots).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
