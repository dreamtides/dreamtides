use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};

/// Ensure the git worktree has no pending changes.
pub fn ensure_clean_worktree(path: &Path) -> Result<()> {
    let status = self::status_porcelain(path)?;
    anyhow::ensure!(status.trim().is_empty(), "Working tree is not clean in {path:?}:\n{status}");

    Ok(())
}

/// Return `git status --porcelain` output.
pub fn status_porcelain(path: &Path) -> Result<String> {
    self::git_output(path, &["status", "--porcelain"])
}

/// Create a new worktree for the agent branch.
pub fn worktree_add(repo_root: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(branch)
        .arg(worktree_path)
        .arg("master")
        .status()
        .with_context(|| format!("Failed to add worktree {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git worktree add failed for {worktree_path:?}");

    Ok(())
}

/// Remove the agent worktree from the repository.
pub fn worktree_remove(repo_root: &Path, worktree_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("worktree")
        .arg("remove")
        .arg(worktree_path)
        .status()
        .with_context(|| format!("Failed to remove worktree {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git worktree remove failed for {worktree_path:?}");

    Ok(())
}

/// Force remove the agent worktree from the repository.
pub fn worktree_remove_force(repo_root: &Path, worktree_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(worktree_path)
        .status()
        .with_context(|| format!("Failed to remove worktree {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git worktree remove failed for {worktree_path:?}");

    Ok(())
}

/// Delete the agent branch.
pub fn branch_delete(repo_root: &Path, branch: &str) -> Result<()> {
    self::git_run(repo_root, &["branch", "-d", branch])
}

/// Fetch the latest master branch.
pub fn fetch_master(repo_root: &Path) -> Result<()> {
    self::git_run(repo_root, &["fetch", "origin", "master"])
}

/// Fetch a revision from another local repo.
pub fn fetch_from(repo_root: &Path, source: &Path, revision: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("fetch")
        .arg(source)
        .arg(revision)
        .status()
        .with_context(|| format!("Failed to fetch {revision} from {source:?} in {repo_root:?}"))?;

    anyhow::ensure!(status.success(), "git fetch failed for {revision} in {repo_root:?}");

    Ok(())
}

/// Start a rebase of the current branch onto origin/master.
pub fn rebase_onto_master(worktree_path: &Path) -> Result<ExitStatus> {
    self::git_status(worktree_path, &["rebase", "origin/master"])
}

/// Start a rebase of the current branch onto another branch.
pub fn rebase_onto_branch(worktree_path: &Path, branch: &str) -> Result<ExitStatus> {
    self::git_status(worktree_path, &["rebase", branch])
}

/// Continue an in-progress rebase.
pub fn rebase_continue(worktree_path: &Path) -> Result<ExitStatus> {
    self::git_status(worktree_path, &["rebase", "--continue"])
}

/// Check if a rebase is in progress.
pub fn rebase_in_progress(worktree_path: &Path) -> Result<bool> {
    let rebase_merge =
        self::git_output(worktree_path, &["rev-parse", "--git-path", "rebase-merge"])?;
    let rebase_apply =
        self::git_output(worktree_path, &["rev-parse", "--git-path", "rebase-apply"])?;

    let merge_path = self::resolve_git_path(worktree_path, rebase_merge.trim());
    let apply_path = self::resolve_git_path(worktree_path, rebase_apply.trim());

    Ok(merge_path.exists() || apply_path.exists())
}

/// Return `git diff master...<branch>` output.
pub fn diff_master_agent(worktree_path: &Path, branch: &str) -> Result<String> {
    let range = format!("master...{branch}");
    self::git_output(worktree_path, &["diff", &range])
}

/// Run difftastic for `git diff master...<branch>`.
pub fn diff_master_agent_difftastic(worktree_path: &Path, branch: &str) -> Result<()> {
    let range = format!("master...{branch}");
    let status = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("-c")
        .arg("diff.external=difft")
        .arg("diff")
        .arg(&range)
        .status()
        .with_context(|| format!("Failed to diff {range} in {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git diff failed for {range} in {worktree_path:?}");

    Ok(())
}

/// Resolve a revision to a commit hash.
pub fn rev_parse(repo_root: &Path, revision: &str) -> Result<String> {
    Ok(self::git_output(repo_root, &["rev-parse", revision])?.trim().to_string())
}

/// Return `git diff` output for unstaged changes.
pub fn diff_worktree(worktree_path: &Path) -> Result<String> {
    self::git_output(worktree_path, &["diff"])
}

/// Run difftastic for unstaged changes.
pub fn diff_worktree_difftastic(worktree_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("-c")
        .arg("diff.external=difft")
        .arg("diff")
        .status()
        .with_context(|| format!("Failed to diff worktree {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git diff failed for worktree {worktree_path:?}");

    Ok(())
}

/// Return `git diff --cached` output for staged changes.
pub fn diff_cached(worktree_path: &Path) -> Result<String> {
    self::git_output(worktree_path, &["diff", "--cached"])
}

/// Run difftastic for staged changes.
pub fn diff_cached_difftastic(worktree_path: &Path) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("-c")
        .arg("diff.external=difft")
        .arg("diff")
        .arg("--cached")
        .status()
        .with_context(|| format!("Failed to diff cached changes in {worktree_path:?}"))?;

    anyhow::ensure!(status.success(), "git diff --cached failed for {worktree_path:?}");

    Ok(())
}

/// Return the number of commits for a revision range.
pub fn rev_list_count(worktree_path: &Path, range: &str) -> Result<usize> {
    let output = self::git_output(worktree_path, &["rev-list", "--count", range])?;
    let trimmed = output.trim();
    let count: usize =
        trimmed.parse().with_context(|| format!("Failed to parse rev-list count {trimmed:?}"))?;

    Ok(count)
}

/// Checkout the master branch at the repository root.
pub fn checkout_master(repo_root: &Path) -> Result<()> {
    self::git_run(repo_root, &["checkout", "master"])
}

/// Checkout a branch at the repository root.
pub fn checkout_branch(repo_root: &Path, branch: &str) -> Result<()> {
    self::git_run(repo_root, &["checkout", branch])
}

/// Fast-forward merge the agent branch into master.
pub fn merge_ff_only(repo_root: &Path, branch: &str) -> Result<()> {
    self::git_run(repo_root, &["merge", "--ff-only", branch])
}

/// Create or update a branch to a revision.
pub fn branch_force(repo_root: &Path, branch: &str, revision: &str) -> Result<()> {
    self::git_run(repo_root, &["branch", "-f", branch, revision])
}

/// Read the configured origin URL.
pub fn remote_origin_url(repo_root: &Path) -> Result<String> {
    Ok(self::git_output(repo_root, &["config", "--get", "remote.origin.url"])?.trim().to_string())
}

fn git_run(repo_root: &Path, args: &[&str]) -> Result<()> {
    let status = self::git_status(repo_root, args)?;
    anyhow::ensure!(status.success(), "git command failed: git -C {repo_root:?} {args:?}");

    Ok(())
}

fn git_status(repo_root: &Path, args: &[&str]) -> Result<ExitStatus> {
    Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run git {args:?} in {repo_root:?}"))
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {args:?} in {repo_root:?}"))?;

    anyhow::ensure!(output.status.success(), "git command failed: git -C {repo_root:?} {args:?}");

    String::from_utf8(output.stdout)
        .with_context(|| format!("git output was not UTF-8 for git -C {repo_root:?} {args:?}"))
}

fn resolve_git_path(worktree_path: &Path, git_path: &str) -> PathBuf {
    let path = Path::new(git_path);
    if path.is_absolute() { path.to_path_buf() } else { worktree_path.join(path) }
}
