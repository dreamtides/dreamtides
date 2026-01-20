//! Claim operations for task management.
//!
//! This module provides operations for claiming, releasing, and querying task
//! claims. Claims are local to the machine and stored in `.lattice/claims/`
//! under the project root (gitignored).

use std::path::Path;

use tracing::{debug, info, warn};

use crate::claim::claim_storage;
use crate::claim::claim_storage::{ClaimData, WriteClaimResult};
use crate::cli::command_dispatch::LatticeResult;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;

/// A claim entry with its associated task ID.
#[derive(Debug, Clone)]
pub struct ClaimEntry {
    /// The Lattice ID of the claimed task.
    pub id: LatticeId,
    /// The claim data (timestamp and work path).
    pub data: ClaimData,
}

/// Claims a task by creating a claim file.
///
/// Creates a claim file for the given task ID, recording the current timestamp
/// and the worktree path from which the task was claimed.
///
/// # Errors
///
/// Returns an error if:
/// - The task is already claimed (claim file exists)
/// - The claim file cannot be written (permission denied, disk full, etc.)
pub fn claim_task(repo_root: &Path, id: &LatticeId, work_path: &Path) -> LatticeResult<()> {
    let claim_path = claim_storage::claim_file_path(repo_root, id)?;
    let claim = ClaimData::new(work_path.to_path_buf());

    // Use exclusive write to avoid TOCTOU race conditions
    match claim_storage::write_claim_exclusive(&claim_path, &claim)? {
        WriteClaimResult::Created => {
            info!(id = %id, work_path = %work_path.display(), "Claimed task");
            Ok(())
        }
        WriteClaimResult::AlreadyExists => {
            warn!(id = %id, path = %claim_path.display(), "Task is already claimed");
            Err(LatticeError::OperationNotAllowed {
                reason: format!("Task {} is already claimed", id),
            })
        }
    }
}

/// Releases a claim by deleting the claim file.
///
/// This operation is idempotent: it succeeds whether or not the claim exists.
/// If the claim doesn't exist, no action is taken.
///
/// # Errors
///
/// Returns an error only for I/O failures (permission denied, etc.).
pub fn release_claim(repo_root: &Path, id: &LatticeId) -> LatticeResult<()> {
    let claim_path = claim_storage::claim_file_path(repo_root, id)?;
    claim_storage::delete_claim(&claim_path)?;
    debug!(id = %id, "Released claim (if it existed)");
    Ok(())
}

/// Checks if a task is currently claimed.
///
/// Returns `true` if a claim file exists for the given task ID, `false`
/// otherwise.
pub fn is_claimed(repo_root: &Path, id: &LatticeId) -> LatticeResult<bool> {
    let claim_path = claim_storage::claim_file_path(repo_root, id)?;
    let exists = claim_path.exists();
    debug!(id = %id, claimed = exists, "Checked claim status");
    Ok(exists)
}

/// Retrieves claim data for a specific task.
///
/// Returns `Ok(Some(data))` if the task is claimed, `Ok(None)` if not claimed.
///
/// # Errors
///
/// Returns an error if the claim file exists but cannot be read or parsed.
pub fn get_claim(repo_root: &Path, id: &LatticeId) -> LatticeResult<Option<ClaimData>> {
    let claim_path = claim_storage::claim_file_path(repo_root, id)?;
    claim_storage::read_claim(&claim_path)
}

/// Lists all claims for the given repository.
///
/// Returns a list of claim entries sorted by `claimed_at` timestamp (oldest
/// first).
///
/// # Errors
///
/// Returns an error if the claims directory cannot be read.
pub fn list_claims(repo_root: &Path) -> LatticeResult<Vec<ClaimEntry>> {
    let claims_dir = claim_storage::claim_dir_path(repo_root)?;

    if !claims_dir.exists() {
        debug!(path = %claims_dir.display(), "Claims directory does not exist");
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&claims_dir).map_err(|e| {
        warn!(path = %claims_dir.display(), error = %e, "Failed to read claims directory");
        LatticeError::ReadError { path: claims_dir.clone(), reason: e.to_string() }
    })?;

    let mut claims = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            warn!(path = %claims_dir.display(), error = %e, "Failed to read directory entry");
            LatticeError::ReadError { path: claims_dir.clone(), reason: e.to_string() }
        })?;

        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Some(claim_entry) = parse_claim_file(&path)?
        {
            claims.push(claim_entry);
        }
    }

    claims.sort_by(|a, b| a.data.claimed_at.cmp(&b.data.claimed_at));
    debug!(count = claims.len(), "Listed claims");
    Ok(claims)
}

/// Parses a claim file and extracts the task ID and claim data.
///
/// Returns `None` if the filename doesn't match the expected pattern.
fn parse_claim_file(path: &Path) -> LatticeResult<Option<ClaimEntry>> {
    let Some(id_str) = path.file_stem().and_then(|s| s.to_str()) else {
        debug!(path = %path.display(), "Skipping file with no stem");
        return Ok(None);
    };

    let id = match LatticeId::parse(id_str) {
        Ok(id) => id,
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Skipping claim file with invalid ID");
            return Ok(None);
        }
    };

    match claim_storage::read_claim(path)? {
        Some(data) => Ok(Some(ClaimEntry { id, data })),
        None => Ok(None),
    }
}
