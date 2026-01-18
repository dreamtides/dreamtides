use std::path::{Path, PathBuf};

use chrono::Utc;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::document_move_ops;
use crate::cli::task_args::ReopenArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::{document_reader, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::index::document_types::UpdateBuilder;
use crate::task::closed_directory;

/// Executes the `lat reopen` command.
///
/// Reopens closed tasks by moving them from `.closed/` subdirectory back to
/// their original location. Updates all links pointing to the reopened task
/// to reflect the new path. Clears `closed-at` timestamp.
pub fn execute(context: CommandContext, args: ReopenArgs) -> LatticeResult<()> {
    info!(
        ids = ?args.ids,
        dry_run = args.dry_run,
        "Executing reopen command"
    );

    let mut results = Vec::new();

    for id_str in &args.ids {
        let result = reopen_single_task(&context, id_str, &args)?;
        results.push(result);
    }

    print_output(&context, &results, args.dry_run);

    info!(count = results.len(), "Reopen command complete");
    Ok(())
}

/// Result of reopening a single task.
struct ReopenResult {
    id: String,
    old_path: String,
    new_path: String,
    links_updated: usize,
}

/// Reopens a single task by ID.
fn reopen_single_task(
    context: &CommandContext,
    id_str: &str,
    args: &ReopenArgs,
) -> LatticeResult<ReopenResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id_str)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id_str.to_string() })?;

    validate_can_reopen(&doc_row.path, doc_row.task_type.is_some())?;

    let old_path = PathBuf::from(&doc_row.path);
    let new_path = closed_directory::unclosed_path_for(&old_path)?;

    info!(
        id = id_str,
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "Reopening task"
    );

    if args.dry_run {
        let links_updated = document_move_ops::count_incoming_links(context, id_str)?;
        return Ok(ReopenResult {
            id: id_str.to_string(),
            old_path: doc_row.path,
            new_path: new_path.to_string_lossy().to_string(),
            links_updated,
        });
    }

    validate_target_available(context, &new_path)?;

    update_document_content(context, &old_path)?;
    document_move_ops::move_document(context, &old_path, &new_path)?;
    info!(
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "File moved from .closed directory"
    );
    let links_updated =
        document_move_ops::rewrite_incoming_links(context, id_str, &old_path, &new_path)?;
    update_index(context, id_str, &new_path)?;

    info!(id = id_str, links_updated, "Task reopened successfully");

    Ok(ReopenResult {
        id: id_str.to_string(),
        old_path: doc_row.path,
        new_path: new_path.to_string_lossy().to_string(),
        links_updated,
    })
}

/// Validates that a task can be reopened.
fn validate_can_reopen(path: &str, is_task: bool) -> LatticeResult<()> {
    if !is_task {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Cannot reopen a knowledge base document (no task-type)".to_string(),
        });
    }

    if !closed_directory::is_in_closed(path) {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Task is not closed (not in a .closed/ directory)".to_string(),
        });
    }

    Ok(())
}

/// Validates that the target path does not already exist.
fn validate_target_available(context: &CommandContext, new_path: &Path) -> LatticeResult<()> {
    let abs_path = context.repo_root.join(new_path);
    if abs_path.exists() {
        return Err(LatticeError::PathAlreadyExists { path: new_path.to_path_buf() });
    }
    Ok(())
}

/// Updates the document content: clears closed-at and updates updated-at.
fn update_document_content(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    let file_path = context.repo_root.join(path);
    let document = document_reader::read(&file_path)?;

    let mut frontmatter = document.frontmatter.clone();
    frontmatter.closed_at = None;
    frontmatter.updated_at = Some(Utc::now());

    let content = frontmatter_parser::format_document(&frontmatter, &document.body)?;
    document_writer::write_raw(&file_path, &content, &WriteOptions::default())?;

    debug!(path = %path.display(), "Document content updated: cleared closed-at timestamp");
    Ok(())
}

/// Updates the index entry for the reopened task.
fn update_index(context: &CommandContext, id: &str, new_path: &Path) -> LatticeResult<()> {
    let new_path_str = new_path.to_string_lossy();
    let now = Utc::now();

    let builder =
        UpdateBuilder::new().path(&new_path_str).is_closed(false).closed_at(None).updated_at(now);

    document_queries::update(&context.conn, id, &builder)?;

    debug!(id, new_path = %new_path.display(), "Index updated for reopened task");
    Ok(())
}

/// Prints output in the appropriate format.
fn print_output(context: &CommandContext, results: &[ReopenResult], dry_run: bool) {
    if context.global.json {
        let json_results: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "old_path": r.old_path,
                    "new_path": r.new_path,
                    "links_updated": r.links_updated,
                    "dry_run": dry_run,
                })
            })
            .collect();

        let output = if results.len() == 1 {
            json_results.into_iter().next().unwrap_or_default()
        } else {
            serde_json::json!(json_results)
        };

        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        let prefix = if dry_run { "[dry-run] " } else { "" };
        for result in results {
            println!("{}Reopened {} -> {}", prefix, result.id, result.new_path);
            if result.links_updated > 0 {
                println!("  {} link(s) updated", result.links_updated);
            }
        }
    }
}
