#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::logging::git as logging_git;

/// Information about a git worktree
#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub is_detached: bool,
}

/// Result of a rebase operation
#[derive(Debug, Clone, PartialEq)]
pub struct RebaseResult {
    pub success: bool,
    pub conflicts: Vec<String>,
}

/// Creates a new worktree at the specified path, checking out the given branch
pub fn create_worktree(repo: &Path, branch: &str, worktree_path: &Path) -> Result<()> {
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("worktree")
        .arg("add")
        .arg(worktree_path)
        .arg(branch)
        .output()
        .context("Failed to execute git worktree add")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to create worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let after = logging_git::capture_state(worktree_path).ok();

    match &result {
        Ok(_) => {
            tracing::info!(
                operation = "git_operation",
                operation_type = "worktree_create",
                repo_path = %repo.display(),
                worktree_path = %worktree_path.display(),
                branch,
                duration_ms,
                ?after,
                result = "success",
                "Created git worktree"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "worktree_create",
                repo_path = %repo.display(),
                worktree_path = %worktree_path.display(),
                branch,
                duration_ms,
                result = "error",
                error = %e,
                "Failed to create worktree"
            );
        }
    }

    result
}

/// Removes the worktree at the specified path
pub fn remove_worktree(repo: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).arg("worktree").arg("remove");

    if force {
        cmd.arg("--force");
    }

    let output =
        cmd.arg(worktree_path).output().context("Failed to execute git worktree remove")?;

    if !output.status.success() {
        bail!("Failed to remove worktree: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Lists all worktrees in the repository
pub fn list_worktrees(repo: &Path) -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git worktree list")?;

    if !output.status.success() {
        bail!("Failed to list worktrees: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&stdout)
}

/// Checks if a worktree exists at the specified path
pub fn worktree_exists(worktree_path: &Path) -> bool {
    worktree_path.join(".git").exists()
}

/// Creates a new branch at the specified start point
pub fn create_branch(repo: &Path, name: &str, start_point: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("branch")
        .arg(name)
        .arg(start_point)
        .output()
        .context("Failed to execute git branch")?;

    if !output.status.success() {
        bail!("Failed to create branch: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Deletes a branch, optionally forcing deletion
pub fn delete_branch(repo: &Path, name: &str, force: bool) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("branch")
        .arg(if force { "-D" } else { "-d" })
        .arg(name)
        .output()
        .context("Failed to execute git branch -d")?;

    if !output.status.success() {
        bail!("Failed to delete branch: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Checks if a branch exists
pub fn branch_exists(repo: &Path, name: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("refs/heads/{name}"))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Gets the current branch name for a worktree
pub fn get_current_branch(worktree: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        bail!("Failed to get current branch: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Gets the commit SHA for a specific ref
pub fn get_head_commit_of_ref(repo: &Path, ref_name: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("rev-parse")
        .arg(ref_name)
        .output()
        .context("Failed to execute git rev-parse")?;

    if !output.status.success() {
        bail!("Failed to get commit for {}: {}", ref_name, String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Gets the HEAD commit SHA for a worktree
pub fn get_head_commit(worktree: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .context("Failed to execute git rev-parse HEAD")?;

    if !output.status.success() {
        bail!("Failed to get HEAD commit: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Gets the merge-base between two refs in a worktree
pub fn get_merge_base(worktree: &Path, ref1: &str, ref2: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("merge-base")
        .arg(ref1)
        .arg(ref2)
        .output()
        .context("Failed to execute git merge-base")?;

    if !output.status.success() {
        bail!("Failed to get merge-base: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Checks if the worktree has commits ahead of the given ref
pub fn has_commits_ahead_of(worktree: &Path, base_ref: &str) -> Result<bool> {
    let head = get_head_commit(worktree)?;
    let merge_base = get_merge_base(worktree, "HEAD", base_ref)?;

    Ok(head != merge_base)
}

/// Checks if there are uncommitted changes in the worktree
pub fn has_uncommitted_changes(worktree: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        bail!("Failed to check uncommitted changes: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(!output.stdout.is_empty())
}

/// Gets the commit message for a specific SHA
pub fn get_commit_message(worktree: &Path, sha: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("log")
        .arg("-1")
        .arg("--format=%B")
        .arg(sha)
        .output()
        .context("Failed to execute git log")?;

    if !output.status.success() {
        bail!("Failed to get commit message: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Strips agent attribution patterns from commit messages
pub fn strip_agent_attribution(message: &str) -> String {
    let patterns = ["🤖 Generated with [Claude Code]", "Generated with [Claude Code]"];

    let result = patterns.iter().fold(message.to_string(), |acc, pattern| acc.replace(pattern, ""));

    let co_authored_regex = Regex::new(r"\n*Co-Authored-By: Claude[^\n]*").unwrap();
    co_authored_regex.replace_all(&result, "").trim().to_string()
}

/// Amends uncommitted changes to the most recent commit
pub fn amend_uncommitted_changes(worktree: &Path) -> Result<()> {
    let add_output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("add")
        .arg("-A")
        .output()
        .context("Failed to execute git add -A")?;

    if !add_output.status.success() {
        bail!("Failed to stage changes: {}", String::from_utf8_lossy(&add_output.stderr));
    }

    let amend_output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("commit")
        .arg("--amend")
        .arg("--no-edit")
        .output()
        .context("Failed to execute git commit --amend")?;

    if !amend_output.status.success() {
        bail!("Failed to amend commit: {}", String::from_utf8_lossy(&amend_output.stderr));
    }

    Ok(())
}

/// Rebases the worktree onto the target branch
pub fn rebase_onto(worktree: &Path, target: &str) -> Result<RebaseResult> {
    let before = logging_git::capture_state(worktree).ok();
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg(target)
        .output()
        .context("Failed to execute git rebase")?;

    let result = if output.status.success() {
        Ok(RebaseResult { success: true, conflicts: vec![] })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            let conflicts = get_conflicted_files(worktree)?;
            Ok(RebaseResult { success: false, conflicts })
        } else {
            Err(anyhow::anyhow!("Rebase failed: {stderr}"))
        }
    };

    let after = logging_git::capture_state(worktree).ok();
    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(rebase_result) if rebase_result.success => {
            tracing::info!(
                operation = "git_operation",
                operation_type = "rebase",
                repo_path = %worktree.display(),
                target,
                duration_ms,
                ?before,
                ?after,
                result = "success",
                "Git rebase succeeded"
            );
        }
        Ok(rebase_result) => {
            tracing::warn!(
                operation = "git_operation",
                operation_type = "rebase",
                repo_path = %worktree.display(),
                target,
                duration_ms,
                ?before,
                ?after,
                result = "conflict",
                conflicts = ?rebase_result.conflicts,
                "Git rebase has conflicts"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "rebase",
                repo_path = %worktree.display(),
                target,
                duration_ms,
                ?before,
                ?after,
                result = "error",
                error = %e,
                "Git rebase failed"
            );
        }
    }

    result
}

/// Checks if a rebase is currently in progress
pub fn is_rebase_in_progress(worktree: &Path) -> bool {
    worktree.join(".git/rebase-merge").exists() || worktree.join(".git/rebase-apply").exists()
}

/// Gets the list of files with merge conflicts
pub fn get_conflicted_files(worktree: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("diff")
        .arg("--name-only")
        .arg("--diff-filter=U")
        .output()
        .context("Failed to execute git diff")?;

    if !output.status.success() {
        bail!("Failed to get conflicted files: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.lines().map(str::to_string).collect())
}

/// Aborts an in-progress rebase
pub fn abort_rebase(worktree: &Path) -> Result<()> {
    let before = logging_git::capture_state(worktree).ok();
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg("--abort")
        .output()
        .context("Failed to execute git rebase --abort")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to abort rebase: {}", String::from_utf8_lossy(&output.stderr)))
    };

    let after = logging_git::capture_state(worktree).ok();
    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::warn!(
                operation = "git_operation",
                operation_type = "rebase_abort",
                repo_path = %worktree.display(),
                duration_ms,
                ?before,
                ?after,
                result = "success",
                "Aborted rebase"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "rebase_abort",
                repo_path = %worktree.display(),
                duration_ms,
                ?before,
                ?after,
                result = "error",
                error = %e,
                "Failed to abort rebase"
            );
        }
    }

    result
}

/// Continues an in-progress rebase
pub fn continue_rebase(worktree: &Path) -> Result<()> {
    let before = logging_git::capture_state(worktree).ok();
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg("--continue")
        .output()
        .context("Failed to execute git rebase --continue")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to continue rebase: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let after = logging_git::capture_state(worktree).ok();
    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::info!(
                operation = "git_operation",
                operation_type = "rebase_continue",
                repo_path = %worktree.display(),
                duration_ms,
                ?before,
                ?after,
                result = "success",
                "Continued rebase successfully"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "rebase_continue",
                repo_path = %worktree.display(),
                duration_ms,
                ?before,
                ?after,
                result = "error",
                error = %e,
                "Failed to continue rebase"
            );
        }
    }

    result
}

/// Squashes all commits since base into a single commit
pub fn squash_commits(worktree: &Path, base: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("reset")
        .arg("--soft")
        .arg(base)
        .output()
        .context("Failed to execute git reset --soft")?;

    if !output.status.success() {
        bail!("Failed to squash commits: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Performs a fast-forward merge of the specified branch
pub fn fast_forward_merge(repo: &Path, branch: &str) -> Result<()> {
    let before = logging_git::capture_state(repo).ok();
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("merge")
        .arg("--ff-only")
        .arg(branch)
        .output()
        .context("Failed to execute git merge --ff-only")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to fast-forward merge: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let after = logging_git::capture_state(repo).ok();
    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::info!(
                operation = "git_operation",
                operation_type = "merge_ff",
                repo_path = %repo.display(),
                branch,
                duration_ms,
                ?before,
                ?after,
                result = "success",
                "Fast-forward merge succeeded"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "merge_ff",
                repo_path = %repo.display(),
                branch,
                duration_ms,
                ?before,
                ?after,
                result = "error",
                error = %e,
                "Fast-forward merge failed"
            );
        }
    }

    result
}

/// Fetches from origin
pub fn fetch_origin(repo: &Path) -> Result<()> {
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("fetch")
        .arg("origin")
        .output()
        .context("Failed to execute git fetch origin")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to fetch from origin: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::debug!(
                operation = "git_operation",
                operation_type = "fetch",
                repo_path = %repo.display(),
                remote = "origin",
                duration_ms,
                result = "success",
                "Fetched from origin"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "fetch",
                repo_path = %repo.display(),
                remote = "origin",
                duration_ms,
                result = "error",
                error = %e,
                "Failed to fetch from origin"
            );
        }
    }

    result
}

/// Fetches a specific ref from a local repository
pub fn fetch_from_local(target_repo: &Path, source_repo: &Path, ref_name: &str) -> Result<()> {
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(target_repo)
        .arg("fetch")
        .arg(source_repo)
        .arg(ref_name)
        .output()
        .context("Failed to execute git fetch from local repository")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to fetch {} from {}: {}",
            ref_name,
            source_repo.display(),
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::debug!(
                operation = "git_operation",
                operation_type = "fetch_local",
                target_repo = %target_repo.display(),
                source_repo = %source_repo.display(),
                ref_name,
                duration_ms,
                result = "success",
                "Fetched from local repository"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "fetch_local",
                target_repo = %target_repo.display(),
                source_repo = %source_repo.display(),
                ref_name,
                duration_ms,
                result = "error",
                error = %e,
                "Failed to fetch from local repository"
            );
        }
    }

    result
}

/// Checks out a branch
pub fn checkout_branch(repo: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("checkout")
        .arg(branch)
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        bail!("Failed to checkout {}: {}", branch, String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Resets the current branch to the specified ref (hard reset)
pub fn reset_to_ref(repo: &Path, ref_name: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("reset")
        .arg("--hard")
        .arg(ref_name)
        .output()
        .context("Failed to execute git reset")?;

    if !output.status.success() {
        bail!("Failed to reset to {}: {}", ref_name, String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Pushes master branch to origin
pub fn push_master_to_origin(repo: &Path) -> Result<()> {
    let before = logging_git::capture_state(repo).ok();
    let start = std::time::Instant::now();

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("push")
        .arg("origin")
        .arg("master")
        .output()
        .context("Failed to execute git push origin master")?;

    let result = if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to push master to origin: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::info!(
                operation = "git_operation",
                operation_type = "push",
                repo_path = %repo.display(),
                branch = "master",
                remote = "origin",
                duration_ms,
                ?before,
                result = "success",
                "Pushed master to origin"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation",
                operation_type = "push",
                repo_path = %repo.display(),
                branch = "master",
                remote = "origin",
                duration_ms,
                ?before,
                result = "error",
                error = %e,
                "Failed to push master to origin"
            );
        }
    }

    result
}

/// Verifies a commit exists on origin/master (after fetching)
pub fn verify_commit_on_origin(repo: &Path, commit_sha: &str) -> Result<()> {
    fetch_origin(repo)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(commit_sha)
        .arg("origin/master")
        .output()
        .context("Failed to verify commit on origin")?;

    if !output.status.success() {
        bail!(
            "CRITICAL: Commit {} is not on origin/master. The push may have failed silently.",
            commit_sha
        );
    }

    Ok(())
}

/// Pulls with rebase from origin/master
pub fn pull_rebase(worktree: &Path) -> Result<()> {
    // Fetch latest changes from origin
    let fetch_output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("fetch")
        .arg("origin")
        .arg("master")
        .output()
        .context("Failed to execute git fetch")?;

    if !fetch_output.status.success() {
        bail!("Failed to fetch from origin: {}", String::from_utf8_lossy(&fetch_output.stderr));
    }

    // Rebase current branch onto origin/master
    let rebase_output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg("origin/master")
        .output()
        .context("Failed to execute git rebase")?;

    if !rebase_output.status.success() {
        bail!(
            "Failed to rebase onto origin/master: {}",
            String::from_utf8_lossy(&rebase_output.stderr)
        );
    }

    Ok(())
}

fn parse_worktree_list(output: &str) -> Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current_path = None;
    let mut current_branch = None;
    let mut is_detached = false;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            if let Some(path) = current_path.take() {
                worktrees.push(WorktreeInfo {
                    path,
                    branch: current_branch.take().unwrap_or_else(|| "HEAD".to_string()),
                    is_detached,
                });
                is_detached = false;
            }
            current_path = Some(line.strip_prefix("worktree ").unwrap().to_string());
        } else if line.starts_with("branch ") {
            current_branch = line
                .strip_prefix("branch ")
                .map(|s| s.strip_prefix("refs/heads/").unwrap_or(s).to_string());
        } else if line.starts_with("detached") {
            is_detached = true;
        }
    }

    if let Some(path) = current_path {
        worktrees.push(WorktreeInfo {
            path,
            branch: current_branch.unwrap_or_else(|| "HEAD".to_string()),
            is_detached,
        });
    }

    Ok(worktrees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_agent_attribution() {
        let message = "Fix bug\n\nCo-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>";
        assert_eq!(strip_agent_attribution(message), "Fix bug");

        let message2 = "Add feature\n\n🤖 Generated with [Claude Code]";
        assert_eq!(strip_agent_attribution(message2), "Add feature");

        let message3 = "Simple commit";
        assert_eq!(strip_agent_attribution(message3), "Simple commit");
    }

    #[test]
    fn test_parse_worktree_list() {
        let output = "worktree /path/to/repo
HEAD abc123

worktree /path/to/worktree1
branch refs/heads/feature

worktree /path/to/worktree2
HEAD def456
detached
";

        let result = parse_worktree_list(output).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].path, "/path/to/repo");
        assert_eq!(result[0].branch, "HEAD");
        assert!(!result[0].is_detached);
        assert_eq!(result[1].path, "/path/to/worktree1");
        assert_eq!(result[1].branch, "feature");
        assert!(!result[1].is_detached);
        assert_eq!(result[2].path, "/path/to/worktree2");
        assert_eq!(result[2].branch, "HEAD");
        assert!(result[2].is_detached);
    }
}
