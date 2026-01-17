use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tracing::{debug, trace};

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;
use crate::index::index_metadata;

/// Classification of changes detected between the last indexed commit and the
/// current repository state.
///
/// This enum provides a quick summary of what type of changes exist, allowing
/// the reconciliation coordinator to choose the appropriate sync strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeStatus {
    /// No changes detected; index is in sync.
    ///
    /// HEAD has not changed since the last indexed commit and there are no
    /// uncommitted changes to markdown files. The fast path applies.
    NoChanges,

    /// Only committed changes exist (HEAD has changed).
    ///
    /// There are changes between the last indexed commit and HEAD, but no
    /// uncommitted changes in the working tree.
    CommittedChanges,

    /// Only uncommitted changes exist (working tree modified).
    ///
    /// HEAD matches the last indexed commit, but there are uncommitted changes
    /// to markdown files (staged or unstaged).
    UncommittedChanges,

    /// Both committed and uncommitted changes exist.
    ///
    /// HEAD has changed since the last indexed commit AND there are uncommitted
    /// changes to markdown files.
    Both,
}

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
/// * `repo_root` - Path to the repository root for checking file existence
///
/// # Errors
///
/// Returns `LatticeError` if git operations fail or database queries fail.
pub fn detect_changes(
    git: &dyn GitOps,
    conn: &Connection,
    repo_root: &Path,
) -> Result<ChangeInfo, LatticeError> {
    let last_indexed_commit = index_metadata::get_last_commit(conn)?;
    let current_head = git.rev_parse("HEAD").ok();

    debug!(?last_indexed_commit, ?current_head, "Checking for changes since last index");

    // Fast path check: if HEAD matches last indexed commit and no uncommitted
    // changes
    if let (Some(last), Some(current)) = (&last_indexed_commit, &current_head)
        && last == current
    {
        trace!("HEAD unchanged, checking for uncommitted changes");
        let uncommitted = get_uncommitted_markdown_files(git)?;
        if uncommitted.is_empty() {
            debug!("Fast path: no changes detected");
            return Ok(ChangeInfo {
                modified_files: Vec::new(),
                deleted_files: Vec::new(),
                uncommitted_files: Vec::new(),
                current_head: current_head.clone(),
                last_indexed_commit: last_indexed_commit.clone(),
            });
        }
        debug!(uncommitted_count = uncommitted.len(), "Found uncommitted changes only");
        return Ok(ChangeInfo {
            modified_files: Vec::new(),
            deleted_files: Vec::new(),
            uncommitted_files: uncommitted,
            current_head: current_head.clone(),
            last_indexed_commit: last_indexed_commit.clone(),
        });
    }

    // Get modified/deleted files since last indexed commit
    let (modified_files, deleted_files) =
        if let (Some(last), Some(current)) = (&last_indexed_commit, &current_head) {
            get_changed_files(git, last, current, repo_root)?
        } else {
            debug!("No last indexed commit; triggering full rebuild path");
            // No last commit means we need full rebuild (return empty to trigger it)
            (Vec::new(), Vec::new())
        };

    let uncommitted_files = get_uncommitted_markdown_files(git)?;

    debug!(
        modified_count = modified_files.len(),
        deleted_count = deleted_files.len(),
        uncommitted_count = uncommitted_files.len(),
        "Change detection complete"
    );

    Ok(ChangeInfo {
        modified_files,
        deleted_files,
        uncommitted_files,
        current_head,
        last_indexed_commit,
    })
}

/// Gets markdown files changed between two commits, distinguishing between
/// modified and deleted files.
///
/// This function queries git for files changed between two commits and then
/// checks each file's existence on the filesystem to determine whether it was
/// modified/added or deleted.
///
/// # Arguments
///
/// * `git` - Git operations trait object
/// * `from_commit` - The starting commit (typically last indexed commit)
/// * `to_commit` - The ending commit (typically HEAD)
/// * `repo_root` - Repository root for checking file existence
///
/// # Returns
///
/// A tuple of (modified_files, deleted_files) where:
/// - `modified_files` includes both newly added and modified files
/// - `deleted_files` includes files that no longer exist
pub fn get_changed_files(
    git: &dyn GitOps,
    from_commit: &str,
    to_commit: &str,
    repo_root: &Path,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), LatticeError> {
    trace!(from_commit, to_commit, "Getting changed files between commits");

    let changed = git.diff(from_commit, to_commit, "*.md")?;

    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    for path in changed {
        let full_path = repo_root.join(&path);
        if full_path.exists() {
            trace!(?path, "File modified/added");
            modified.push(path);
        } else {
            trace!(?path, "File deleted");
            deleted.push(path);
        }
    }

    debug!(
        modified_count = modified.len(),
        deleted_count = deleted.len(),
        "Categorized changed files"
    );

    Ok((modified, deleted))
}

/// Gets markdown files with uncommitted changes (staged or unstaged).
///
/// This function queries `git status` for any markdown files that have been
/// modified but not yet committed. This includes:
/// - Staged changes (in the index)
/// - Unstaged changes (in the working tree)
/// - Untracked markdown files
///
/// # Arguments
///
/// * `git` - Git operations trait object
///
/// # Returns
///
/// A list of paths (relative to repo root) with uncommitted changes.
pub fn get_uncommitted_changes(git: &dyn GitOps) -> Result<Vec<PathBuf>, LatticeError> {
    get_uncommitted_markdown_files(git)
}

impl ChangeInfo {
    /// Returns the classification of changes detected.
    ///
    /// This provides a quick summary for the reconciliation coordinator to
    /// determine the sync strategy without inspecting individual file lists.
    pub fn status(&self) -> ChangeStatus {
        let has_committed = self.has_committed_changes();
        let has_uncommitted = self.has_uncommitted_changes();

        match (has_committed, has_uncommitted) {
            (false, false) => ChangeStatus::NoChanges,
            (true, false) => ChangeStatus::CommittedChanges,
            (false, true) => ChangeStatus::UncommittedChanges,
            (true, true) => ChangeStatus::Both,
        }
    }

    /// Returns true if no changes were detected (fast path).
    ///
    /// The fast path applies when:
    /// - HEAD has not changed since the last indexed commit
    /// - There are no uncommitted changes to markdown files
    pub fn is_fast_path(&self) -> bool {
        self.status() == ChangeStatus::NoChanges
    }

    /// Returns true if there are committed changes between last indexed commit
    /// and HEAD.
    fn has_committed_changes(&self) -> bool {
        // If commits differ, there are committed changes
        if self.current_head != self.last_indexed_commit {
            return true;
        }
        // If commits are the same but we have modified/deleted files, that's a
        // data inconsistency that should be treated as having changes
        !self.modified_files.is_empty() || !self.deleted_files.is_empty()
    }

    /// Returns true if there are uncommitted changes to markdown files.
    fn has_uncommitted_changes(&self) -> bool {
        !self.uncommitted_files.is_empty()
    }

    /// Returns the total number of files that need to be processed.
    pub fn total_changes(&self) -> usize {
        self.modified_files.len() + self.deleted_files.len() + self.uncommitted_files.len()
    }
}

/// Internal implementation of uncommitted file detection.
fn get_uncommitted_markdown_files(git: &dyn GitOps) -> Result<Vec<PathBuf>, LatticeError> {
    let statuses = git.status("*.md")?;
    let paths: Vec<PathBuf> = statuses.into_iter().map(|s| s.path).collect();
    trace!(count = paths.len(), "Found uncommitted markdown files");
    Ok(paths)
}
