use std::path::Path;

use rusqlite::Connection;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::ready_command::{ready_filter, ready_output};
use crate::cli::workflow_args::ReadyArgs;
use crate::document::document_reader;
use crate::index::{document_queries, label_queries};
use crate::task::ready_calculator::{self, ReadyTask};

/// Executes the `lat ready` command.
///
/// Shows work available to start: tasks that are not closed, have all blockers
/// closed, and (by default) are not claimed and not P4.
pub fn execute(context: CommandContext, args: ReadyArgs) -> LatticeResult<()> {
    info!("Executing ready command");

    // Build filter from CLI arguments
    let filter = ready_filter::build_filter(&context.conn, &args)?;

    // Query ready tasks
    let tasks = ready_calculator::query_ready_tasks(&context.conn, &context.repo_root, &filter)?;

    info!(count = tasks.len(), "Found ready tasks");

    // Calculate counts
    let claimed_count = ready_output::count_claimed(&tasks);
    let open_count = tasks.len() - claimed_count;

    // Output based on format
    if context.global.json {
        output_json(&context, &tasks)?;
    } else if args.pretty {
        ready_output::output_pretty(&tasks, claimed_count, open_count);
    } else {
        ready_output::output_text(&tasks, claimed_count);
    }

    Ok(())
}

/// Loads additional data and outputs JSON format.
fn output_json(context: &CommandContext, tasks: &[ReadyTask]) -> LatticeResult<()> {
    let mut bodies = Vec::with_capacity(tasks.len());
    let mut labels_list = Vec::with_capacity(tasks.len());
    let mut parents = Vec::with_capacity(tasks.len());

    for task in tasks {
        let doc = &task.document;

        // Load body content
        let body = load_body_content(&context.repo_root, &doc.path)?;
        bodies.push(body);

        // Load labels
        let labels = label_queries::get_labels(&context.conn, &doc.id)?;
        labels_list.push(labels);

        // Load parent info
        let parent = load_parent_info(&context.conn, doc.parent_id.as_deref())?;
        parents.push(parent);
    }

    ready_output::output_json(tasks, &bodies, &labels_list, &parents)
}

/// Loads the markdown body content from the filesystem.
fn load_body_content(repo_root: &Path, relative_path: &str) -> LatticeResult<String> {
    let full_path = repo_root.join(relative_path);
    let doc = document_reader::read(&full_path)?;
    Ok(doc.body)
}

/// Loads parent document info (ID and description) if available.
fn load_parent_info(
    conn: &Connection,
    parent_id: Option<&str>,
) -> LatticeResult<Option<(String, String)>> {
    match parent_id {
        Some(id) => {
            let doc = document_queries::lookup_by_id(conn, id)?;
            Ok(doc.map(|d| (d.id, d.description)))
        }
        None => Ok(None),
    }
}
