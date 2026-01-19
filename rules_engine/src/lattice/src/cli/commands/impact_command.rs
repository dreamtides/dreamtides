use std::collections::{BTreeMap, HashSet, VecDeque};

use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::ImpactArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_types::DocumentRow;
use crate::index::{document_queries, link_queries};

/// Maximum depth to traverse when analyzing impact.
///
/// Prevents excessive traversal in highly-connected graphs. Documents beyond
/// this depth are omitted from results.
const MAX_DEPTH: usize = 10;

/// Executes the `lat impact` command.
///
/// Analyzes what documents would be affected by changes to the specified
/// document. Uses breadth-first traversal of incoming links (backlinks) to
/// find all documents that reference the target, directly or transitively.
///
/// Output is grouped by distance from the source document:
/// - Depth 1: Documents that directly link to the target
/// - Depth 2: Documents that link to depth-1 documents
/// - And so on...
#[instrument(skip_all, name = "impact_command", fields(id = %args.id))]
pub fn execute(context: CommandContext, args: ImpactArgs) -> LatticeResult<()> {
    info!(id = %args.id, "Executing impact command");

    let source_doc = document_queries::lookup_by_id(&context.conn, &args.id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id.clone() })?;
    debug!(id = %args.id, path = %source_doc.path, "Source document found");

    let impact_result = analyze_impact(&context, &args.id)?;
    info!(
        total_affected = impact_result.total_count,
        max_depth = impact_result.max_depth,
        "Impact analysis completed"
    );

    if context.global.json {
        output_json(&source_doc, &impact_result)?;
    } else {
        output_text(&source_doc, &impact_result);
    }

    Ok(())
}

/// Result of impact analysis.
#[derive(Debug)]
struct ImpactResult {
    /// Documents grouped by their distance from the source.
    /// Key is the depth (1 = direct reference, 2 = one hop away, etc.)
    by_depth: BTreeMap<usize, Vec<DocumentRow>>,
    /// Total number of affected documents.
    total_count: usize,
    /// Maximum depth reached in the traversal.
    max_depth: usize,
}

/// JSON output format for an affected document.
#[derive(Debug, Serialize)]
struct ImpactDocumentJson {
    id: String,
    name: String,
    description: String,
    path: String,
    depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    is_closed: bool,
}

/// JSON output format for the full impact result.
#[derive(Debug, Serialize)]
struct ImpactResultJson {
    source: SourceDocumentJson,
    total_affected: usize,
    max_depth: usize,
    affected_documents: Vec<ImpactDocumentJson>,
}

/// JSON output format for the source document.
#[derive(Debug, Serialize)]
struct SourceDocumentJson {
    id: String,
    name: String,
    description: String,
    path: String,
}

/// Analyzes the impact of changes to a document using BFS traversal.
///
/// Traverses incoming links (backlinks) starting from the source document.
/// Tracks visited documents to handle circular references without infinite
/// loops. Documents are grouped by their distance from the source.
fn analyze_impact(context: &CommandContext, source_id: &str) -> LatticeResult<ImpactResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut by_depth: BTreeMap<usize, Vec<DocumentRow>> = BTreeMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    visited.insert(source_id.to_string());
    queue.push_back((source_id.to_string(), 0));

    while let Some((current_id, current_depth)) = queue.pop_front() {
        if current_depth >= MAX_DEPTH {
            debug!(id = %current_id, depth = current_depth, "Reached max depth, skipping");
            continue;
        }

        let backlink_ids = link_queries::get_source_ids(&context.conn, &current_id)?;
        debug!(
            id = %current_id,
            depth = current_depth,
            backlink_count = backlink_ids.len(),
            "Processing document"
        );

        for backlink_id in backlink_ids {
            if visited.contains(&backlink_id) {
                continue;
            }

            visited.insert(backlink_id.clone());
            let next_depth = current_depth + 1;

            if let Some(doc) = document_queries::lookup_by_id(&context.conn, &backlink_id)? {
                by_depth.entry(next_depth).or_default().push(doc);
                queue.push_back((backlink_id, next_depth));
            } else {
                debug!(id = %backlink_id, "Backlink document not found in index, skipping");
            }
        }
    }

    let total_count: usize = by_depth.values().map(Vec::len).sum();
    let max_depth = by_depth.keys().max().copied().unwrap_or(0);

    Ok(ImpactResult { by_depth, total_count, max_depth })
}

/// Outputs the impact analysis in text format.
fn output_text(source_doc: &DocumentRow, result: &ImpactResult) {
    let source_name = color_theme::bold(&source_doc.name);

    if result.total_count == 0 {
        println!("No documents would be affected by changes to {}.", source_name);
        return;
    }

    let count_str = output_format::format_count(result.total_count, "document", "documents");
    println!("{count_str} would be affected by changes to {}:", source_name);
    println!();

    for (depth, docs) in &result.by_depth {
        let depth_label = format_depth_label(*depth);
        println!("{}", color_theme::accent(&depth_label));

        for doc in docs {
            print_document(doc);
        }
        println!();
    }
}

/// Formats the depth label for text output.
fn format_depth_label(depth: usize) -> String {
    match depth {
        1 => "Direct references (depth 1):".to_string(),
        _ => format!("Depth {} (transitive):", depth),
    }
}

/// Prints a single document in text format.
fn print_document(doc: &DocumentRow) {
    let id_str = color_theme::lattice_id(&doc.id);
    let type_str = format_type_priority(doc);
    let name_str = color_theme::bold(&doc.name);
    let status_str =
        if doc.is_closed { color_theme::muted("[closed]").to_string() } else { String::new() };

    println!("  {id_str} {type_str} {name_str} {status_str}");
    println!("    {}", doc.description);
    println!("    {}", color_theme::path(&doc.path));
}

/// Formats task type and priority for display.
fn format_type_priority(doc: &DocumentRow) -> String {
    match (doc.task_type, doc.priority) {
        (Some(task_type), Some(priority)) => {
            color_theme::task_type(format!("[{}/P{}]", task_type, priority)).to_string()
        }
        (Some(task_type), None) => color_theme::task_type(format!("[{}]", task_type)).to_string(),
        (None, _) => color_theme::muted("[doc]").to_string(),
    }
}

/// Outputs the impact analysis in JSON format.
fn output_json(source_doc: &DocumentRow, result: &ImpactResult) -> LatticeResult<()> {
    let source = SourceDocumentJson {
        id: source_doc.id.clone(),
        name: source_doc.name.clone(),
        description: source_doc.description.clone(),
        path: source_doc.path.clone(),
    };

    let mut affected_documents = Vec::with_capacity(result.total_count);
    for (depth, docs) in &result.by_depth {
        for doc in docs {
            affected_documents.push(ImpactDocumentJson {
                id: doc.id.clone(),
                name: doc.name.clone(),
                description: doc.description.clone(),
                path: doc.path.clone(),
                depth: *depth,
                task_type: doc.task_type.map(|t| t.to_string()),
                priority: doc.priority,
                is_closed: doc.is_closed,
            });
        }
    }

    let json_result = ImpactResultJson {
        source,
        total_affected: result.total_count,
        max_depth: result.max_depth,
        affected_documents,
    };

    let json_str = output_format::output_json(&json_result).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
