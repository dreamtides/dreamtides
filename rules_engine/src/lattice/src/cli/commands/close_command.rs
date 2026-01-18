use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use tracing::{debug, info, warn};

use crate::claim::claim_operations;
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::CloseArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::{document_reader, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::index::document_types::UpdateBuilder;
use crate::index::{document_queries, link_queries};
use crate::link::link_extractor::{self, ExtractedLink};
use crate::link::link_normalization::link_analysis::{self, NormalizationAction};
use crate::link::link_normalization::link_transforms::{self, LinkTransform};
use crate::link::link_resolver;
use crate::task::closed_directory;

/// Executes the `lat close` command.
///
/// Closes tasks by moving them to `.closed/` subdirectory. Updates all links
/// pointing to the closed task to reflect the new path. Sets `closed-at`
/// timestamp and releases any claims.
pub fn execute(context: CommandContext, args: CloseArgs) -> LatticeResult<()> {
    info!(
        ids = ?args.ids,
        reason = ?args.reason,
        dry_run = args.dry_run,
        "Executing close command"
    );

    let mut results = Vec::new();

    for id_str in &args.ids {
        let result = close_single_task(&context, id_str, &args)?;
        results.push(result);
    }

    print_output(&context, &results, args.dry_run);

    info!(count = results.len(), "Close command complete");
    Ok(())
}

/// Result of closing a single task.
struct CloseResult {
    id: String,
    old_path: String,
    new_path: String,
    links_updated: usize,
}

/// Closes a single task by ID.
fn close_single_task(
    context: &CommandContext,
    id_str: &str,
    args: &CloseArgs,
) -> LatticeResult<CloseResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id_str)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id_str.to_string() })?;

    validate_can_close(&doc_row.path, doc_row.task_type.is_some())?;

    let old_path = PathBuf::from(&doc_row.path);
    let new_path = closed_directory::closed_path_for(&old_path)?;

    info!(
        id = id_str,
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "Closing task"
    );

    if args.dry_run {
        let links_updated = count_incoming_links(context, id_str)?;
        return Ok(CloseResult {
            id: id_str.to_string(),
            old_path: doc_row.path,
            new_path: new_path.to_string_lossy().to_string(),
            links_updated,
        });
    }

    let parent_dir = old_path.parent().ok_or_else(|| LatticeError::InvalidPath {
        path: old_path.clone(),
        reason: "Task path has no parent directory".to_string(),
    })?;
    closed_directory::ensure_closed_dir(parent_dir, &context.repo_root)?;

    update_document_content(context, &old_path, args.reason.as_deref())?;
    move_file(context, &old_path, &new_path)?;
    let links_updated = rewrite_incoming_links(context, id_str, &old_path, &new_path)?;
    update_index(context, id_str, &new_path)?;
    release_claim(context, id_str)?;

    info!(id = id_str, links_updated, "Task closed successfully");

    Ok(CloseResult {
        id: id_str.to_string(),
        old_path: doc_row.path,
        new_path: new_path.to_string_lossy().to_string(),
        links_updated,
    })
}

/// Validates that a task can be closed.
fn validate_can_close(path: &str, is_task: bool) -> LatticeResult<()> {
    if !is_task {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Cannot close a knowledge base document (no task-type)".to_string(),
        });
    }

    if closed_directory::is_in_closed(path) {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Task is already closed".to_string(),
        });
    }

    Ok(())
}

/// Updates the document content with closed-at timestamp and optional reason.
fn update_document_content(
    context: &CommandContext,
    path: &Path,
    reason: Option<&str>,
) -> LatticeResult<()> {
    let file_path = context.repo_root.join(path);
    let document = document_reader::read(&file_path)?;

    let mut frontmatter = document.frontmatter.clone();
    frontmatter.closed_at = Some(Utc::now());
    frontmatter.updated_at = Some(Utc::now());

    let new_body = match reason {
        Some(text) => append_closure_reason(&document.body, text),
        None => document.body.clone(),
    };

    let content = frontmatter_parser::format_document(&frontmatter, &new_body)?;
    document_writer::write_raw(&file_path, &content, &WriteOptions::default())?;

    debug!(path = %path.display(), "Document content updated with closed-at timestamp");
    Ok(())
}

/// Appends a closure reason to the document body.
fn append_closure_reason(body: &str, reason: &str) -> String {
    let trimmed = body.trim_end();
    if trimmed.is_empty() {
        format!("## Closure Reason\n\n{}\n", reason)
    } else {
        format!("{}\n\n## Closure Reason\n\n{}\n", trimmed, reason)
    }
}

/// Moves a file from old_path to new_path.
fn move_file(context: &CommandContext, old_path: &Path, new_path: &Path) -> LatticeResult<()> {
    let abs_old = context.repo_root.join(old_path);
    let abs_new = context.repo_root.join(new_path);

    fs::rename(&abs_old, &abs_new).map_err(|e| LatticeError::WriteError {
        path: new_path.to_path_buf(),
        reason: format!("Failed to move file: {}", e),
    })?;

    info!(
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "File moved to .closed directory"
    );

    Ok(())
}

/// Counts incoming links to a document (for dry-run output).
fn count_incoming_links(context: &CommandContext, id: &str) -> LatticeResult<usize> {
    let links = link_queries::query_incoming(&context.conn, id)?;
    Ok(links.len())
}

/// Rewrites all incoming links to point to the new path.
fn rewrite_incoming_links(
    context: &CommandContext,
    target_id: &str,
    old_target_path: &Path,
    new_target_path: &Path,
) -> LatticeResult<usize> {
    let incoming_links = link_queries::query_incoming(&context.conn, target_id)?;
    let mut total_updated = 0;

    let source_ids: Vec<_> = incoming_links.iter().map(|l| l.source_id.as_str()).collect();
    let unique_sources: std::collections::BTreeSet<_> = source_ids.into_iter().collect();

    for source_id in unique_sources {
        let source_row = document_queries::lookup_by_id(&context.conn, source_id)?;
        let Some(source_row) = source_row else {
            warn!(source_id, "Source document not found for link rewriting");
            continue;
        };

        let updated =
            rewrite_links_in_document(context, &source_row.path, old_target_path, new_target_path)?;
        if updated > 0 {
            total_updated += updated;
        }
    }

    debug!(target_id, total_updated, "Incoming links rewritten");
    Ok(total_updated)
}

/// Rewrites links in a single document that point to the moved target.
fn rewrite_links_in_document(
    context: &CommandContext,
    source_path_str: &str,
    old_target_path: &Path,
    new_target_path: &Path,
) -> LatticeResult<usize> {
    let source_path = PathBuf::from(source_path_str);
    let file_path = context.repo_root.join(&source_path);
    let document = document_reader::read(&file_path)?;

    let extracted = link_extractor::extract(&document.body);
    let transforms =
        build_transforms_for_move(&source_path, &extracted.links, old_target_path, new_target_path);

    if transforms.is_empty() {
        return Ok(0);
    }

    let result = link_transforms::apply_transforms(&document.body, &transforms);

    if result.modified_count > 0 {
        document_writer::update_body(&file_path, &result.content, &WriteOptions::with_timestamp())?;
        debug!(
            source_path = source_path_str,
            modified_count = result.modified_count,
            "Links rewritten in document"
        );
    }

    Ok(result.modified_count)
}

/// Builds link transforms for links that point to the moved target.
fn build_transforms_for_move(
    source_path: &Path,
    links: &[ExtractedLink],
    old_target_path: &Path,
    new_target_path: &Path,
) -> Vec<LinkTransform> {
    let source_dir = source_path.parent().unwrap_or(Path::new(""));

    let old_relative = link_resolver::relative_path_between(source_dir, old_target_path);
    let new_relative = link_resolver::relative_path_between(source_dir, new_target_path);

    links
        .iter()
        .filter_map(|link| {
            let link_path = link.path.as_ref()?;
            if link_analysis::normalize_path(link_path)
                == link_analysis::normalize_path(&old_relative.to_string_lossy())
            {
                Some(LinkTransform {
                    link: link.clone(),
                    action: NormalizationAction::UpdatePath {
                        new_relative_path: new_relative.to_string_lossy().to_string(),
                    },
                })
            } else {
                None
            }
        })
        .collect()
}

/// Updates the index entry for the closed task.
fn update_index(context: &CommandContext, id: &str, new_path: &Path) -> LatticeResult<()> {
    let new_path_str = new_path.to_string_lossy();
    let now = Utc::now();

    let builder = UpdateBuilder::new()
        .path(&new_path_str)
        .is_closed(true)
        .closed_at(Some(now))
        .updated_at(now);

    document_queries::update(&context.conn, id, &builder)?;

    debug!(id, new_path = %new_path.display(), "Index updated for closed task");
    Ok(())
}

/// Releases any claim on the task.
fn release_claim(context: &CommandContext, id: &str) -> LatticeResult<()> {
    let lattice_id = id.parse().map_err(|_| LatticeError::MalformedId { value: id.to_string() })?;
    claim_operations::release_claim(&context.repo_root, &lattice_id)?;
    debug!(id, "Claim released (if any)");
    Ok(())
}

/// Prints output in the appropriate format.
fn print_output(context: &CommandContext, results: &[CloseResult], dry_run: bool) {
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
            println!("{}Closed {} -> {}", prefix, result.id, result.new_path);
            if result.links_updated > 0 {
                println!("  {} link(s) updated", result.links_updated);
            }
        }
    }
}
