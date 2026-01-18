use std::path::Path;

use chrono::{Duration, Utc};
use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::claim::claim_operations::ClaimEntry;
use crate::claim::{claim_operations, claim_storage};
use crate::cli::command_dispatch::LatticeResult;
use crate::config::config_schema::ClaimConfig;
use crate::index::document_queries;

/// Reason why a claim is considered stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    /// The task is in a `.closed/` directory.
    TaskClosed,
    /// The task file no longer exists in the repository.
    TaskNotFound,
    /// The worktree path from the claim no longer exists.
    WorkPathNotFound,
    /// The claim is older than the configured threshold.
    AgeExceeded { days: u32 },
}

/// Result of checking a claim for staleness.
#[derive(Debug)]
pub enum StalenessCheck {
    /// The claim is stale and should be removed.
    Stale(StaleReason),
    /// The claim is still valid.
    Active,
}

/// Summary of a cleanup operation.
#[derive(Debug, Default)]
pub struct CleanupSummary {
    /// Claims that were released because they were stale.
    pub released: Vec<(String, StaleReason)>,
    /// Claims that were kept because they are still active.
    pub kept: Vec<String>,
    /// Claims that could not be processed due to errors.
    pub errors: Vec<(String, String)>,
}

/// Checks if a single claim is stale.
///
/// Evaluates the claim against staleness criteria in order:
/// 1. Task is in a `.closed/` directory (checked via index)
/// 2. Task file no longer exists (not in index)
/// 3. `work_path` no longer exists on filesystem
/// 4. Claim age exceeds threshold
///
/// Returns `StalenessCheck::Stale` with the first matching reason, or
/// `StalenessCheck::Active` if the claim is still valid.
pub fn is_claim_stale(
    conn: &Connection,
    entry: &ClaimEntry,
    config: &ClaimConfig,
) -> LatticeResult<StalenessCheck> {
    let id_str = entry.id.as_str();
    debug!(id = id_str, "Checking claim staleness");

    // Check if the task exists and its closed status via the index
    match document_queries::lookup_by_id(conn, id_str)? {
        Some(doc_row) => {
            if doc_row.is_closed {
                debug!(id = id_str, "Task is closed");
                return Ok(StalenessCheck::Stale(StaleReason::TaskClosed));
            }
        }
        None => {
            debug!(id = id_str, "Task not found in index");
            return Ok(StalenessCheck::Stale(StaleReason::TaskNotFound));
        }
    }

    // Check if the work path still exists
    if !entry.data.work_path.exists() {
        debug!(
            id = id_str,
            work_path = %entry.data.work_path.display(),
            "Work path no longer exists"
        );
        return Ok(StalenessCheck::Stale(StaleReason::WorkPathNotFound));
    }

    // Check if the claim is older than the threshold
    let stale_days = config.stale_days;
    let age_threshold = Duration::days(i64::from(stale_days));
    let age = Utc::now() - entry.data.claimed_at;

    if age > age_threshold {
        debug!(
            id = id_str,
            age_days = age.num_days(),
            threshold_days = stale_days,
            "Claim exceeds age threshold"
        );
        return Ok(StalenessCheck::Stale(StaleReason::AgeExceeded { days: stale_days }));
    }

    debug!(id = id_str, "Claim is active");
    Ok(StalenessCheck::Active)
}

/// Cleans up stale claims for the given repository.
///
/// Scans all claims in the repository's claims directory, checks each against
/// staleness criteria, and deletes stale claim files.
///
/// Returns a summary of the cleanup operation.
pub fn cleanup_stale_claims(
    conn: &Connection,
    repo_root: &Path,
    config: &ClaimConfig,
) -> LatticeResult<CleanupSummary> {
    let claims = claim_operations::list_claims(repo_root)?;
    let count = claims.len();

    if count == 0 {
        debug!("No claims to check");
        return Ok(CleanupSummary::default());
    }

    info!(count, "Checking claims for staleness");
    let mut summary = CleanupSummary::default();

    for entry in claims {
        let id_str = entry.id.as_str().to_string();

        match is_claim_stale(conn, &entry, config) {
            Ok(StalenessCheck::Stale(reason)) => {
                let claim_path = claim_storage::claim_file_path(repo_root, &entry.id)?;

                if let Err(e) = claim_storage::delete_claim(&claim_path) {
                    warn!(
                        id = %id_str,
                        error = %e,
                        "Failed to delete stale claim"
                    );
                    summary.errors.push((id_str, e.to_string()));
                } else {
                    info!(id = %id_str, reason = %reason, "Released stale claim");
                    summary.released.push((id_str, reason));
                }
            }
            Ok(StalenessCheck::Active) => {
                debug!(id = %id_str, "Claim is active");
                summary.kept.push(id_str);
            }
            Err(e) => {
                warn!(id = %id_str, error = %e, "Error checking claim staleness");
                summary.errors.push((id_str, e.to_string()));
            }
        }
    }

    info!(
        released = summary.released.len(),
        kept = summary.kept.len(),
        errors = summary.errors.len(),
        "Claim cleanup complete"
    );

    Ok(summary)
}

impl std::fmt::Display for StaleReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaleReason::TaskClosed => write!(f, "task closed"),
            StaleReason::TaskNotFound => write!(f, "task not found"),
            StaleReason::WorkPathNotFound => write!(f, "work path no longer exists"),
            StaleReason::AgeExceeded { days } => write!(f, "older than {days} days"),
        }
    }
}

impl CleanupSummary {
    /// Returns the total number of claims processed.
    pub fn total(&self) -> usize {
        self.released.len() + self.kept.len() + self.errors.len()
    }
}
