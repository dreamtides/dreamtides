use std::path::Path;
use std::time::Instant;

use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;
use crate::index::reconciliation::change_detection::ChangeInfo;
use crate::index::reconciliation::{change_detection, sync_strategies};
use crate::index::{connection_pool, schema_definition};

/// Outcome of a reconciliation operation.
///
/// This enum describes what action the reconciliation coordinator took to bring
/// the index into sync with the git repository state. Each variant provides
/// relevant statistics about the operation performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationResult {
    /// No changes detected; index was already in sync.
    ///
    /// The fast path determined that HEAD has not changed since the last index
    /// update and there are no uncommitted changes to markdown files.
    Skipped,

    /// Incremental sync completed successfully.
    ///
    /// Only changed files were re-indexed, preserving unchanged entries.
    Incremental {
        /// Number of files that were updated or added.
        files_updated: usize,
        /// Number of files that were removed from the index.
        files_removed: usize,
    },

    /// Full rebuild completed successfully.
    ///
    /// The entire index was deleted and recreated from scratch. This happens
    /// when the index is missing, has a schema version mismatch, or when an
    /// error during incremental sync triggers a fallback.
    FullRebuild {
        /// Total number of documents indexed.
        documents_indexed: usize,
    },
}

/// Synchronizes the SQLite index with the current git repository state.
///
/// This is the main entry point for index reconciliation, called at the start
/// of every `lat` command. The coordinator determines the best strategy to use:
///
/// 1. **Fast path (skip)**: If HEAD is unchanged and there are no uncommitted
///    changes to markdown files, no work is needed.
///
/// 2. **Incremental sync**: If only some files have changed, update just those
///    entries while preserving the rest of the index.
///
/// 3. **Full rebuild**: If the index doesn't exist, has a schema version
///    mismatch, or an error occurs during incremental sync.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory
/// * `git` - Git operations trait object for interacting with the repository
/// * `conn` - SQLite database connection to the index
///
/// # Errors
///
/// Returns `LatticeError` if:
/// - Database operations fail
/// - Git operations fail
/// - File I/O errors occur during indexing
///
/// If errors occur during incremental sync, a full rebuild is attempted before
/// propagating any errors.
pub fn reconcile(
    repo_root: &Path,
    git: &dyn GitOps,
    conn: &Connection,
) -> Result<ReconciliationResult, LatticeError> {
    let start = Instant::now();
    info!("Starting index reconciliation");

    let result = reconcile_inner(repo_root, git, conn);

    let elapsed = start.elapsed();
    match &result {
        Ok(ReconciliationResult::Skipped) => {
            info!(elapsed_ms = elapsed.as_millis(), "Reconciliation skipped (no changes)");
        }
        Ok(ReconciliationResult::Incremental { files_updated, files_removed }) => {
            info!(
                elapsed_ms = elapsed.as_millis(),
                files_updated, files_removed, "Incremental reconciliation complete"
            );
        }
        Ok(ReconciliationResult::FullRebuild { documents_indexed }) => {
            info!(elapsed_ms = elapsed.as_millis(), documents_indexed, "Full rebuild complete");
        }
        Err(e) => {
            warn!(elapsed_ms = elapsed.as_millis(), error = %e, "Reconciliation failed");
        }
    }

    result
}

/// Inner reconciliation logic, separated for error recovery handling.
fn reconcile_inner(
    repo_root: &Path,
    git: &dyn GitOps,
    conn: &Connection,
) -> Result<ReconciliationResult, LatticeError> {
    // Check if full rebuild is required
    if let Some(reason) = needs_full_rebuild(conn)? {
        info!(reason, "Full rebuild required");
        return perform_full_rebuild(repo_root, git, conn);
    }

    // Try fast path
    let change_info = change_detection::detect_changes(git, conn)?;

    if change_info.is_fast_path() {
        debug!("Fast path: no changes detected");
        return Ok(ReconciliationResult::Skipped);
    }

    // Attempt incremental sync
    match try_incremental_sync(repo_root, git, conn, &change_info) {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!(error = %e, "Incremental sync failed, attempting full rebuild");
            perform_full_rebuild(repo_root, git, conn)
        }
    }
}

/// Checks if a full rebuild is required and returns the reason if so.
fn needs_full_rebuild(conn: &Connection) -> Result<Option<&'static str>, LatticeError> {
    // Check schema version
    match schema_definition::schema_version(conn)? {
        None => {
            return Ok(Some("index has no schema (new or corrupted)"));
        }
        Some(version) if version != schema_definition::SCHEMA_VERSION => {
            debug!(
                index_version = version,
                expected_version = schema_definition::SCHEMA_VERSION,
                "Schema version mismatch"
            );
            return Ok(Some("schema version mismatch"));
        }
        Some(_) => {}
    }

    Ok(None)
}

/// Attempts an incremental sync of changed files.
fn try_incremental_sync(
    repo_root: &Path,
    git: &dyn GitOps,
    conn: &Connection,
    change_info: &ChangeInfo,
) -> Result<ReconciliationResult, LatticeError> {
    debug!(
        modified_count = change_info.modified_files.len(),
        uncommitted_count = change_info.uncommitted_files.len(),
        "Performing incremental sync"
    );

    let result = sync_strategies::incremental_sync(repo_root, git, conn, change_info)?;

    Ok(ReconciliationResult::Incremental {
        files_updated: result.files_updated,
        files_removed: result.files_removed,
    })
}

/// Performs a full index rebuild from scratch.
fn perform_full_rebuild(
    repo_root: &Path,
    git: &dyn GitOps,
    conn: &Connection,
) -> Result<ReconciliationResult, LatticeError> {
    info!("Performing full index rebuild");

    // Delete existing index data
    connection_pool::delete_index(repo_root)?;

    // Create fresh schema
    schema_definition::create_schema(conn)?;

    // Index all documents
    let result = sync_strategies::full_rebuild(repo_root, git, conn)?;

    // Optimize after bulk operation
    connection_pool::optimize(conn)?;
    connection_pool::checkpoint(conn)?;

    Ok(ReconciliationResult::FullRebuild { documents_indexed: result.documents_indexed })
}
