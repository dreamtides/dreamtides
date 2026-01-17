use std::path::Path;

use rusqlite::Connection;

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;
use crate::index::reconciliation::change_detection::ChangeInfo;

/// Result of an incremental sync operation.
#[derive(Debug, Clone)]
pub struct IncrementalResult {
    /// Number of files that were updated or added to the index.
    pub files_updated: usize,
    /// Number of files that were removed from the index.
    pub files_removed: usize,
}

/// Result of a full rebuild operation.
#[derive(Debug, Clone)]
pub struct FullRebuildResult {
    /// Total number of documents indexed.
    pub documents_indexed: usize,
}

/// Performs an incremental sync of changed files.
///
/// This function updates only the documents that have changed since the last
/// index update, preserving unchanged entries for efficiency.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory
/// * `_git` - Git operations trait object (used for file listing)
/// * `_conn` - SQLite database connection to the index
/// * `change_info` - Information about changed files from change detection
///
/// # Errors
///
/// Returns `LatticeError` if database operations or file I/O fails.
pub fn incremental_sync(
    _repo_root: &Path,
    _git: &dyn GitOps,
    _conn: &Connection,
    change_info: &ChangeInfo,
) -> Result<IncrementalResult, LatticeError> {
    // Placeholder implementation - will be completed in a follow-up task
    // For now, return counts based on change_info

    let files_to_update = change_info.modified_files.len() + change_info.uncommitted_files.len();
    let files_to_remove = change_info.deleted_files.len();

    Ok(IncrementalResult { files_updated: files_to_update, files_removed: files_to_remove })
}

/// Performs a full index rebuild from scratch.
///
/// This function deletes all existing index data and re-indexes every markdown
/// document in the repository. Used when the index is missing, has a schema
/// version mismatch, or when incremental sync fails.
///
/// # Arguments
///
/// * `_repo_root` - Path to the repository root directory
/// * `git` - Git operations trait object for listing all tracked files
/// * `_conn` - SQLite database connection to the index
///
/// # Errors
///
/// Returns `LatticeError` if database operations, git operations, or file I/O
/// fails.
pub fn full_rebuild(
    _repo_root: &Path,
    git: &dyn GitOps,
    _conn: &Connection,
) -> Result<FullRebuildResult, LatticeError> {
    // Placeholder implementation - will be completed in a follow-up task
    // For now, count the markdown files

    let all_files = git.ls_files("*.md")?;
    let documents_indexed = all_files.len();

    Ok(FullRebuildResult { documents_indexed })
}
