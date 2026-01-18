//! Shared operations for moving documents and rewriting links.
//!
//! Used by both `close_command` and `reopen_command` for moving tasks between
//! their normal location and `.closed/` directories.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::document::document_reader;
use crate::document::document_writer::{self, WriteOptions};
use crate::error::error_types::LatticeError;
use crate::index::{document_queries, link_queries};
use crate::link::link_extractor::{self, ExtractedLink};
use crate::link::link_normalization::link_analysis::{self, NormalizationAction};
use crate::link::link_normalization::link_transforms::{self, LinkTransform};
use crate::link::link_resolver;

/// Moves a file from old_path to new_path.
pub fn move_document(
    context: &CommandContext,
    old_path: &Path,
    new_path: &Path,
) -> LatticeResult<()> {
    let abs_old = context.repo_root.join(old_path);
    let abs_new = context.repo_root.join(new_path);

    fs::rename(&abs_old, &abs_new).map_err(|e| LatticeError::WriteError {
        path: new_path.to_path_buf(),
        reason: format!("Failed to move file: {}", e),
    })?;

    debug!(
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "Document moved"
    );

    Ok(())
}

/// Counts incoming links to a document (for dry-run output).
pub fn count_incoming_links(context: &CommandContext, id: &str) -> LatticeResult<usize> {
    let links = link_queries::query_incoming(&context.conn, id)?;
    Ok(links.len())
}

/// Rewrites all incoming links to point to the new path.
pub fn rewrite_incoming_links(
    context: &CommandContext,
    target_id: &str,
    old_target_path: &Path,
    new_target_path: &Path,
) -> LatticeResult<usize> {
    let incoming_links = link_queries::query_incoming(&context.conn, target_id)?;
    let mut total_updated = 0;

    let source_ids: Vec<_> = incoming_links.iter().map(|l| l.source_id.as_str()).collect();
    let unique_sources: BTreeSet<_> = source_ids.into_iter().collect();

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
