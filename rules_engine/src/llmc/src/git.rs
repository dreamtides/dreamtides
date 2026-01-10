#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

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
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("worktree")
        .arg("add")
        .arg(worktree_path)
        .arg(branch)
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        bail!("Failed to create worktree: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Removes the worktree at the specified path
pub fn remove_worktree(worktree_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("worktree")
        .arg("remove")
        .arg(worktree_path)
        .output()
        .context("Failed to execute git worktree remove")?;

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

/// Rebases the worktree onto the target branch
pub fn rebase_onto(worktree: &Path, target: &str) -> Result<RebaseResult> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg(target)
        .output()
        .context("Failed to execute git rebase")?;

    if output.status.success() {
        return Ok(RebaseResult { success: true, conflicts: vec![] });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("CONFLICT") || stderr.contains("conflict") {
        let conflicts = get_conflicted_files(worktree)?;
        Ok(RebaseResult { success: false, conflicts })
    } else {
        bail!("Rebase failed: {stderr}");
    }
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
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg("--abort")
        .output()
        .context("Failed to execute git rebase --abort")?;

    if !output.status.success() {
        bail!("Failed to abort rebase: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Continues an in-progress rebase
pub fn continue_rebase(worktree: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("rebase")
        .arg("--continue")
        .output()
        .context("Failed to execute git rebase --continue")?;

    if !output.status.success() {
        bail!("Failed to continue rebase: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
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
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("merge")
        .arg("--ff-only")
        .arg(branch)
        .output()
        .context("Failed to execute git merge --ff-only")?;

    if !output.status.success() {
        bail!("Failed to fast-forward merge: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Fetches from origin
pub fn fetch_origin(repo: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("fetch")
        .arg("origin")
        .output()
        .context("Failed to execute git fetch origin")?;

    if !output.status.success() {
        bail!("Failed to fetch from origin: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Pulls with rebase from the current branch's upstream
pub fn pull_rebase(worktree: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("pull")
        .arg("--rebase")
        .output()
        .context("Failed to execute git pull --rebase")?;

    if !output.status.success() {
        bail!("Failed to pull with rebase: {}", String::from_utf8_lossy(&output.stderr));
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
