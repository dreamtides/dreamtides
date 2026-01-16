use std::path::PathBuf;

use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;

/// Markdown file pattern for git pathspec filtering.
const MARKDOWN_PATTERN: &str = "*.md";

/// Lists all tracked markdown documents in the repository.
///
/// Uses `git ls-files '*.md'` to enumerate files. Excludes gitignored and
/// untracked files automatically.
///
/// # Returns
/// A vector of paths relative to the repository root.
pub fn list_all_documents(git: &dyn GitOps) -> Result<Vec<PathBuf>, LatticeError> {
    let paths = git.ls_files(MARKDOWN_PATTERN)?;
    debug!(count = paths.len(), "enumerated tracked markdown documents");
    Ok(paths)
}

/// Lists markdown documents changed since a specific commit.
///
/// Uses `git diff --name-only <since_commit>..HEAD -- '*.md'` to find
/// files modified between commits. Used for incremental index reconciliation.
///
/// # Arguments
/// * `git` - Git operations implementation
/// * `since_commit` - Git commit hash or reference to compare from
///
/// # Returns
/// A vector of paths relative to the repository root for changed files.
pub fn list_changed_documents(
    git: &dyn GitOps,
    since_commit: &str,
) -> Result<Vec<PathBuf>, LatticeError> {
    let paths = git.diff(since_commit, "HEAD", MARKDOWN_PATTERN)?;
    debug!(
        count = paths.len(),
        since = since_commit,
        "found changed markdown documents since commit"
    );
    Ok(paths)
}

/// Lists uncommitted markdown document changes.
///
/// Uses `git status --porcelain -- '*.md'` to find staged and unstaged
/// modifications. Returns only paths, not the status characters.
///
/// # Returns
/// A vector of paths relative to the repository root for uncommitted files.
pub fn list_uncommitted_changes(git: &dyn GitOps) -> Result<Vec<PathBuf>, LatticeError> {
    let statuses = git.status(MARKDOWN_PATTERN)?;
    let paths: Vec<PathBuf> = statuses.into_iter().map(|s| s.path).collect();
    debug!(count = paths.len(), "found uncommitted markdown changes");
    Ok(paths)
}
