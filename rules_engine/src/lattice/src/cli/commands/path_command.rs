use std::collections::{HashMap, VecDeque};

use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::PathArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_types::DocumentRow;
use crate::index::{document_queries, link_queries};

/// Executes the `lat path` command.
///
/// Finds the shortest path between two documents using breadth-first search
/// on the outgoing links graph. Validates that both provided IDs exist before
/// searching.
#[instrument(skip_all, name = "path_command", fields(id1 = %args.id1, id2 = %args.id2))]
pub fn execute(context: CommandContext, args: PathArgs) -> LatticeResult<()> {
    info!(source = %args.id1, target = %args.id2, "Executing path command");

    let source_doc = document_queries::lookup_by_id(&context.conn, &args.id1)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id1.clone() })?;
    debug!(id = %args.id1, path = %source_doc.path, "Source document found");

    let target_doc = document_queries::lookup_by_id(&context.conn, &args.id2)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id2.clone() })?;
    debug!(id = %args.id2, path = %target_doc.path, "Target document found");

    let path_ids = find_shortest_path(&context, &args.id1, &args.id2)?;
    info!(path_length = path_ids.as_ref().map(Vec::len), "Path search completed");

    if context.global.json {
        output_json(&context, &source_doc, &target_doc, path_ids.as_deref())?;
    } else {
        output_text(&context, &source_doc, &target_doc, path_ids.as_deref())?;
    }

    Ok(())
}

/// Information about a document in the path.
#[derive(Debug, Serialize)]
struct PathNode {
    id: String,
    name: String,
    description: String,
    path: String,
}

/// Result of a path search between two documents.
#[derive(Debug, Serialize)]
struct PathResult {
    source: PathNode,
    target: PathNode,
    path: Vec<PathNode>,
    length: usize,
}

/// Finds the shortest path between two documents using BFS.
///
/// Returns None if no path exists. Returns Some(vec![source_id]) if source ==
/// target. Otherwise returns the path as a sequence of document IDs from source
/// to target.
fn find_shortest_path(
    context: &CommandContext,
    source_id: &str,
    target_id: &str,
) -> LatticeResult<Option<Vec<String>>> {
    if source_id == target_id {
        debug!(id = source_id, "Source equals target, path length 0");
        return Ok(Some(vec![source_id.to_string()]));
    }

    let mut visited: HashMap<String, String> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    visited.insert(source_id.to_string(), String::new());
    queue.push_back(source_id.to_string());

    while let Some(current_id) = queue.pop_front() {
        let neighbors = link_queries::get_target_ids(&context.conn, &current_id)?;
        debug!(id = %current_id, neighbor_count = neighbors.len(), "Exploring document");

        for neighbor_id in neighbors {
            if visited.contains_key(&neighbor_id) {
                continue;
            }

            visited.insert(neighbor_id.clone(), current_id.clone());

            if neighbor_id == target_id {
                debug!(target = target_id, "Target found, reconstructing path");
                return Ok(Some(reconstruct_path(&visited, source_id, target_id)));
            }

            queue.push_back(neighbor_id);
        }
    }

    debug!(source = source_id, target = target_id, "No path found");
    Ok(None)
}

/// Reconstructs the path from source to target using parent pointers.
fn reconstruct_path(
    parents: &HashMap<String, String>,
    source_id: &str,
    target_id: &str,
) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = target_id.to_string();

    while !current.is_empty() {
        path.push(current.clone());
        current = parents.get(&current).cloned().unwrap_or_default();
    }

    path.reverse();

    debug_assert!(
        path.first().is_some_and(|first| first == source_id),
        "Path should start with source"
    );
    debug_assert!(path.last().is_some_and(|last| last == target_id), "Path should end with target");

    path
}

/// Resolves document IDs to PathNode structs with full document information.
fn resolve_path_nodes(
    context: &CommandContext,
    path_ids: &[String],
) -> LatticeResult<Vec<PathNode>> {
    let mut nodes = Vec::with_capacity(path_ids.len());

    for id in path_ids {
        let doc = document_queries::lookup_by_id(&context.conn, id)?.ok_or_else(|| {
            LatticeError::DatabaseError {
                reason: format!("Document {id} in path no longer exists"),
            }
        })?;

        nodes.push(PathNode {
            id: doc.id,
            name: doc.name,
            description: doc.description,
            path: doc.path,
        });
    }

    Ok(nodes)
}

/// Outputs the path result in text format.
fn output_text(
    context: &CommandContext,
    source_doc: &DocumentRow,
    target_doc: &DocumentRow,
    path_ids: Option<&[String]>,
) -> LatticeResult<()> {
    let source_name = color_theme::bold(&source_doc.name);
    let target_name = color_theme::bold(&target_doc.name);

    match path_ids {
        None => {
            println!("No path exists from {} to {}.", source_name, target_name);
        }
        Some(ids) if ids.len() == 1 => {
            println!("{} and {} are the same document.", source_name, target_name);
        }
        Some(ids) => {
            let path_nodes = resolve_path_nodes(context, ids)?;
            let hops = ids.len() - 1;
            let hop_word = if hops == 1 { "hop" } else { "hops" };

            println!("Path from {} to {} ({} {}):", source_name, target_name, hops, hop_word);
            println!();

            for (i, node) in path_nodes.iter().enumerate() {
                let id_str = color_theme::lattice_id(&node.id);
                let name_str = color_theme::bold(&node.name);
                let is_last = i == path_nodes.len() - 1;

                println!("  {id_str} {name_str}");
                if !is_last {
                    println!("    |");
                    println!("    v");
                }
            }
        }
    }

    Ok(())
}

/// Outputs the path result in JSON format.
fn output_json(
    context: &CommandContext,
    source_doc: &DocumentRow,
    target_doc: &DocumentRow,
    path_ids: Option<&[String]>,
) -> LatticeResult<()> {
    let source_node = PathNode {
        id: source_doc.id.clone(),
        name: source_doc.name.clone(),
        description: source_doc.description.clone(),
        path: source_doc.path.clone(),
    };

    let target_node = PathNode {
        id: target_doc.id.clone(),
        name: target_doc.name.clone(),
        description: target_doc.description.clone(),
        path: target_doc.path.clone(),
    };

    let (path_nodes, length) = match path_ids {
        None => (vec![], 0),
        Some(ids) => {
            let nodes = resolve_path_nodes(context, ids)?;
            let len = if ids.len() > 1 { ids.len() - 1 } else { 0 };
            (nodes, len)
        }
    };

    let result = PathResult { source: source_node, target: target_node, path: path_nodes, length };

    let json_str = output_format::output_json(&result).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
