use std::path::PathBuf;

use rusqlite::Connection;

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;
use crate::index::index_metadata;

/// Information about changes detected in the repository.
///
/// This struct captures the results of comparing the current git state against
/// the last indexed commit. It is used by the reconciliation coordinator to
/// determine whether to skip, perform an incremental sync, or trigger a full
/// rebuild.
#[derive(Debug, Clone, Default)]
pub struct ChangeInfo {
    /// Files modified between the last indexed commit and HEAD.
    pub modified_files: Vec<PathBuf>,

    /// Files deleted between the last indexed commit and HEAD.
    pub deleted_files: Vec<PathBuf>,

    /// Files with uncommitted changes (staged or unstaged).
    pub uncommitted_files: Vec<PathBuf>,

    /// The current HEAD commit hash.
    pub current_head: Option<String>,

    /// The last indexed commit hash.
    pub last_indexed_commit: Option<String>,
}

/// Detects changes between the last indexed commit and the current repository
/// state.
///
/// This function compares the current HEAD commit against the last commit that
/// was indexed, collecting information about modified, deleted, and uncommitted
/// markdown files. The returned [`ChangeInfo`] is used by the reconciliation
/// coordinator to determine the appropriate sync strategy.
///
/// # Arguments
///
/// * `git` - Git operations trait object for interacting with the repository
/// * `conn` - SQLite database connection to read the last indexed commit
///
/// # Errors
///
/// Returns `LatticeError` if git operations fail or database queries fail.
pub fn detect_changes(git: &dyn GitOps, conn: &Connection) -> Result<ChangeInfo, LatticeError> {
    let last_indexed_commit = index_metadata::get_last_commit(conn)?;
    let current_head = git.rev_parse("HEAD").ok();

    // Fast path check: if HEAD matches last indexed commit and no uncommitted
    // changes
    if let (Some(last), Some(current)) = (&last_indexed_commit, &current_head)
        && last == current
    {
        // Check for uncommitted changes
        let uncommitted = get_uncommitted_markdown_files(git)?;
        if uncommitted.is_empty() {
            return Ok(ChangeInfo {
                modified_files: Vec::new(),
                deleted_files: Vec::new(),
                uncommitted_files: Vec::new(),
                current_head: current_head.clone(),
                last_indexed_commit: last_indexed_commit.clone(),
            });
        }
    }

    // Get modified files since last indexed commit
    let (modified_files, deleted_files) =
        if let (Some(last), Some(current)) = (&last_indexed_commit, &current_head) {
            get_changed_files_between_commits(git, last, current)?
        } else {
            // No last commit means we need full rebuild (return empty to trigger it)
            (Vec::new(), Vec::new())
        };

    let uncommitted_files = get_uncommitted_markdown_files(git)?;

    Ok(ChangeInfo {
        modified_files,
        deleted_files,
        uncommitted_files,
        current_head,
        last_indexed_commit,
    })
}

impl ChangeInfo {
    /// Returns true if no changes were detected (fast path).
    ///
    /// The fast path applies when:
    /// - HEAD has not changed since the last indexed commit
    /// - There are no uncommitted changes to markdown files
    pub fn is_fast_path(&self) -> bool {
        self.modified_files.is_empty()
            && self.deleted_files.is_empty()
            && self.uncommitted_files.is_empty()
            && self.current_head == self.last_indexed_commit
    }
}

/// Gets markdown files changed between two commits.
fn get_changed_files_between_commits(
    git: &dyn GitOps,
    from_commit: &str,
    to_commit: &str,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), LatticeError> {
    let changed = git.diff(from_commit, to_commit, "*.md")?;

    // For now, treat all changed files as modified
    // A more sophisticated implementation would distinguish added/modified/deleted
    // by checking file existence or using git diff status codes
    Ok((changed, Vec::new()))
}

/// Gets markdown files with uncommitted changes.
fn get_uncommitted_markdown_files(git: &dyn GitOps) -> Result<Vec<PathBuf>, LatticeError> {
    let statuses = git.status("*.md")?;
    Ok(statuses.into_iter().map(|s| s.path).collect())
}
