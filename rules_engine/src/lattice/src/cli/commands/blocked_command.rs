use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::output_format;
use crate::cli::query_args::BlockedArgs;
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;
use crate::task::dependency_graph::DependencyGraph;

/// A blocked task with information about what is blocking it.
#[derive(Debug, Clone)]
pub struct BlockedTask {
    pub document: DocumentRow,
    pub open_blockers: Vec<BlockerInfo>,
}

/// Information about a task that is blocking another task.
#[derive(Debug, Clone, Serialize)]
pub struct BlockerInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: String,
}

/// Executes the `lat blocked` command.
///
/// Shows tasks with unresolved blockers (open tasks in their `blocked-by`
/// field).
#[instrument(skip_all, name = "blocked_command")]
pub fn execute(context: CommandContext, args: BlockedArgs) -> LatticeResult<()> {
    info!("Executing blocked command");

    let blocked_tasks = query_blocked_tasks(&context, &args)?;

    info!(count = blocked_tasks.len(), "Found blocked tasks");

    if context.global.json {
        output_json(&blocked_tasks)?;
    } else {
        output_text(&blocked_tasks, args.show_blockers);
    }

    Ok(())
}

/// JSON output format for the blocked command.
#[derive(Debug, Clone, Serialize)]
struct BlockedTaskJson {
    id: String,
    name: String,
    description: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    blockers: Vec<BlockerInfo>,
}

/// Queries for tasks that are blocked by at least one open task.
fn query_blocked_tasks(
    context: &CommandContext,
    args: &BlockedArgs,
) -> LatticeResult<Vec<BlockedTask>> {
    debug!("Building dependency graph");
    let graph = DependencyGraph::build_from_connection(&context.conn)?;

    let filter = build_filter(args);
    let candidates = document_queries::query(&context.conn, &filter)?;

    debug!(candidates = candidates.len(), "Evaluating candidate tasks for blocked status");

    let mut blocked_tasks = Vec::new();
    for doc in candidates {
        let open_blockers = find_open_blockers(&context.conn, &graph, &doc)?;
        if !open_blockers.is_empty() {
            blocked_tasks.push(BlockedTask { document: doc, open_blockers });
        }
    }

    if let Some(limit) = args.limit {
        blocked_tasks.truncate(limit);
    }

    Ok(blocked_tasks)
}

/// Builds a document filter from CLI arguments.
fn build_filter(args: &BlockedArgs) -> DocumentFilter {
    DocumentFilter {
        include_closed: false,
        in_tasks_dir: Some(true),
        path_prefix: args.path.clone(),
        ..Default::default()
    }
}

/// Finds all open blockers for a given document.
fn find_open_blockers(
    conn: &Connection,
    graph: &DependencyGraph,
    doc: &DocumentRow,
) -> LatticeResult<Vec<BlockerInfo>> {
    let blocker_ids = graph.get_blockers(&doc.id);
    let mut open_blockers = Vec::new();

    for blocker_id in blocker_ids {
        if let Some(blocker_doc) = document_queries::lookup_by_id(conn, &blocker_id)?
            && !blocker_doc.is_closed
        {
            open_blockers.push(BlockerInfo {
                id: blocker_doc.id.clone(),
                name: blocker_doc.name.clone(),
                description: blocker_doc.description.clone(),
                state: "open".to_string(),
            });
        }
    }

    Ok(open_blockers)
}

/// Outputs blocked tasks in text format.
fn output_text(tasks: &[BlockedTask], show_blockers: bool) {
    if tasks.is_empty() {
        println!("No blocked tasks found.");
        return;
    }

    let count_str = output_format::format_count(tasks.len(), "task", "tasks");
    println!("Blocked {count_str}:");

    for task in tasks {
        let doc = &task.document;
        let priority_str = output_format::format_priority(doc.priority.unwrap_or(2));
        println!("  {}: {} - {} [{}]", doc.id, doc.name, doc.description, priority_str);

        if show_blockers {
            let blocker_ids: Vec<&str> = task.open_blockers.iter().map(|b| b.id.as_str()).collect();
            let blockers_str =
                blocker_ids.iter().map(|id| format!("{id} (open)")).collect::<Vec<_>>().join(", ");
            println!("    Blocked by: {blockers_str}");
        }
    }
}

/// Outputs blocked tasks in JSON format.
fn output_json(tasks: &[BlockedTask]) -> LatticeResult<()> {
    let json_tasks: Vec<BlockedTaskJson> = tasks
        .iter()
        .map(|task| BlockedTaskJson {
            id: task.document.id.clone(),
            name: task.document.name.clone(),
            description: task.document.description.clone(),
            path: task.document.path.clone(),
            priority: task.document.priority,
            blockers: task.open_blockers.clone(),
        })
        .collect();

    let json_str = output_format::output_json_array(&json_tasks).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
