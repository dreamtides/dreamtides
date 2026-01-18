use std::fmt;
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::index::link_queries::{self, LinkType};

/// The `.closed/` directory segment that indicates a task is closed.
const CLOSED_DIR_SEGMENT: &str = "/.closed/";

/// Task state computed from filesystem location and dependencies.
///
/// State is determined by two factors:
/// 1. Filesystem location: Tasks in `.closed/` directories are closed
/// 2. Dependencies: Tasks with open blockers are blocked
///
/// There is no `InProgress` state - use the claim system for work tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// Task exists outside any `.closed/` directory with no open blockers.
    ///
    /// Open tasks appear in `lat ready` output (unless claimed or P4 priority).
    Open,

    /// Task has at least one open (non-closed) entry in its `blocked-by` field.
    ///
    /// Blocked tasks do not appear in `lat ready` output.
    Blocked,

    /// Task resides in a `.closed/` subdirectory.
    ///
    /// Typically `tasks/.closed/` under the parent directory.
    Closed,
}

/// Determines if a path represents a closed task.
///
/// A task is closed if its path contains the `/.closed/` segment.
/// This is the fast path check that doesn't require database access.
///
/// # Examples
///
/// ```ignore
/// assert!(is_closed_path("auth/tasks/.closed/fix_bug.md"));
/// assert!(!is_closed_path("auth/tasks/fix_bug.md"));
/// ```
pub fn is_closed_path(path: &str) -> bool {
    path.contains(CLOSED_DIR_SEGMENT)
}

/// Computes task state from path only (fast path).
///
/// This function only checks filesystem location. It cannot detect blocked
/// state since that requires checking the status of blocking tasks in the
/// database. Use [`compute_state_with_blockers`] for complete state
/// computation.
///
/// # Returns
///
/// - `Closed` if path contains `/.closed/`
/// - `Open` otherwise (may actually be blocked, but we can't tell from path)
pub fn state_from_path(path: &str) -> TaskState {
    if is_closed_path(path) { TaskState::Closed } else { TaskState::Open }
}

/// Computes task state with blocker resolution.
///
/// This function provides complete state computation by:
/// 1. Checking if the document is in a `.closed/` directory
/// 2. Querying the `blocked-by` links and checking if any targets are open
///
/// # Arguments
///
/// * `conn` - Database connection for querying blockers
/// * `document_id` - The Lattice ID of the document to check
/// * `is_closed` - Whether the document is in a `.closed/` directory
///   (pre-computed)
///
/// # Returns
///
/// - `Closed` if `is_closed` is true
/// - `Blocked` if any `blocked-by` target is not closed
/// - `Open` otherwise
///
/// # Errors
///
/// Returns `LatticeError` if database queries fail.
pub fn compute_state_with_blockers(
    conn: &Connection,
    document_id: &str,
    is_closed: bool,
) -> Result<TaskState, LatticeError> {
    if is_closed {
        debug!(document_id, "Task is closed (in .closed/ directory)");
        return Ok(TaskState::Closed);
    }

    // Check if any blockers are still open
    let blocked_by_links =
        link_queries::query_outgoing_by_type(conn, document_id, LinkType::BlockedBy)?;

    if blocked_by_links.is_empty() {
        debug!(document_id, "Task has no blockers, state is open");
        return Ok(TaskState::Open);
    }

    for link in blocked_by_links {
        if let Some(blocker) = document_queries::lookup_by_id(conn, &link.target_id)?
            && !blocker.is_closed
        {
            debug!(document_id, blocker_id = blocker.id, "Task blocked by open task");
            return Ok(TaskState::Blocked);
        }
        // If blocker document doesn't exist in index, treat as if closed
        // (possibly pruned or external reference)
    }

    debug!(document_id, "All blockers are closed, state is open");
    Ok(TaskState::Open)
}

/// Computes task state from a filesystem path with blocker resolution.
///
/// Convenience function that combines path-based closed detection with
/// blocker resolution. Use this when you have a path and need complete state.
///
/// # Arguments
///
/// * `conn` - Database connection for querying blockers
/// * `document_id` - The Lattice ID of the document
/// * `path` - The filesystem path to the document
///
/// # Errors
///
/// Returns `LatticeError` if database queries fail.
pub fn compute_state(
    conn: &Connection,
    document_id: &str,
    path: &Path,
) -> Result<TaskState, LatticeError> {
    let path_str = path.to_string_lossy();
    let is_closed = is_closed_path(&path_str);
    compute_state_with_blockers(conn, document_id, is_closed)
}

impl TaskState {
    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Open => "open",
            TaskState::Blocked => "blocked",
            TaskState::Closed => "closed",
        }
    }

    /// Returns true if this state represents a completed task.
    pub fn is_closed(&self) -> bool {
        matches!(self, TaskState::Closed)
    }

    /// Returns true if this task has open blockers.
    pub fn is_blocked(&self) -> bool {
        matches!(self, TaskState::Blocked)
    }

    /// Returns true if this task is available for work.
    ///
    /// Note: This only checks state, not priority or claims.
    pub fn is_open(&self) -> bool {
        matches!(self, TaskState::Open)
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
