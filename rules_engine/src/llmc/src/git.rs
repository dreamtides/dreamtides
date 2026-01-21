use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::logging::git as logging_git;
/// Maximum number of retries for git operations that encounter lock file errors
const GIT_LOCK_RETRY_COUNT: u32 = 5;
/// Delay between retries when a lock file is encountered (milliseconds)
const GIT_LOCK_RETRY_DELAY_MS: u64 = 500;
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
                operation = "git_operation", operation_type = "worktree_create",
                repo_path = % repo.display(), worktree_path = % worktree_path.display(),
                branch, duration_ms, ? after, result = "success", "Created git worktree"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation", operation_type = "worktree_create",
                repo_path = % repo.display(), worktree_path = % worktree_path.display(),
                branch, duration_ms, result = "error", error = % e,
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("git worktree remove failed: {}", stderr);
        if !force {
            tracing::info!("Retrying worktree removal with --force");
            let force_output = Command::new("git")
                .arg("-C")
                .arg(repo)
                .arg("worktree")
                .arg("remove")
                .arg("--force")
                .arg(worktree_path)
                .output()
                .context("Failed to execute git worktree remove --force")?;
            if force_output.status.success() {
                return Ok(());
            }
            tracing::warn!(
                "git worktree remove --force also failed: {}",
                String::from_utf8_lossy(&force_output.stderr)
            );
        }
        let _ = Command::new("git").arg("-C").arg(repo).arg("worktree").arg("prune").output();
        if worktree_path.exists() {
            tracing::info!(
                "Attempting manual removal of worktree directory: {}",
                worktree_path.display()
            );
            std::fs::remove_dir_all(worktree_path).with_context(|| {
                format!("Failed to manually remove worktree directory: {}", worktree_path.display())
            })?;
            let prune_output = Command::new("git")
                .arg("-C")
                .arg(repo)
                .arg("worktree")
                .arg("prune")
                .output()
                .context("Failed to execute git worktree prune")?;
            if !prune_output.status.success() {
                tracing::warn!(
                    "git worktree prune failed: {}",
                    String::from_utf8_lossy(&prune_output.stderr)
                );
            }
        }
    }
    Ok(())
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
/// Checks if `ancestor` is an ancestor of `descendant`
pub fn is_ancestor(worktree: &Path, ancestor: &str, descendant: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(ancestor)
        .arg(descendant)
        .output()
        .context("Failed to execute git merge-base --is-ancestor")?;
    Ok(output.status.success())
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
/// Checks if there are staged changes ready to be committed
pub fn has_staged_changes(worktree: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .output()
        .context("Failed to execute git diff --cached")?;
    Ok(!output.status.success())
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

/// Creates a new commit from uncommitted changes when no prior commit exists
pub fn create_uncommitted_changes_commit(worktree: &Path) -> Result<()> {
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
    let commit_output = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .arg("commit")
        .arg("-m")
        .arg("Work in progress (uncommitted changes from Stop hook)")
        .output()
        .context("Failed to execute git commit")?;
    if !commit_output.status.success() {
        bail!("Failed to create commit: {}", String::from_utf8_lossy(&commit_output.stderr));
    }
    Ok(())
}
pub fn rebase_onto(worktree: &Path, target: &str) -> Result<RebaseResult> {
    let rebase_in_progress = is_rebase_in_progress(worktree);
    tracing::debug!(
        operation = "git_operation", operation_type = "rebase_check", repo_path = %
        worktree.display(), rebase_in_progress,
        "Checking rebase state before starting rebase"
    );
    if rebase_in_progress {
        tracing::info!(
            operation = "git_operation", operation_type = "rebase", repo_path = %
            worktree.display(), target,
            "Stale rebase state detected, cleaning up before starting new rebase"
        );
        if let Err(e) = abort_rebase(worktree) {
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), error = % e,
                "Failed to abort stale rebase, will attempt to proceed anyway"
            );
        }
    }
    let before = logging_git::capture_state(worktree).ok();
    let start = std::time::Instant::now();
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), target, attempt,
                "Retrying rebase after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(worktree)
            .arg("rebase")
            .arg(target)
            .output()
            .context("Failed to execute git rebase")?;
        if output.status.success() {
            let after = logging_git::capture_state(worktree).ok();
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::info!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), target, duration_ms, ? before, ? after, result =
                "success", retries = attempt, "Git rebase succeeded"
            );
            return Ok(RebaseResult { success: true, conflicts: vec![] });
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            let after = logging_git::capture_state(worktree).ok();
            let duration_ms = start.elapsed().as_millis() as u64;
            let conflicts = get_conflicted_files(worktree)?;
            tracing::info!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), target, duration_ms, ? before, ? after, result =
                "conflict", conflicts = ? conflicts, "Git rebase has conflicts"
            );
            return Ok(RebaseResult { success: false, conflicts });
        }
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), target, attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        if stderr.contains("already a rebase-merge directory")
            || stderr.contains("already a rebase-apply directory")
        {
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase", repo_path = %
                worktree.display(), target, attempt,
                "Stale rebase directory detected, attempting force cleanup and retry"
            );
            if let Err(e) = force_cleanup_rebase_state(worktree) {
                tracing::error!(
                    operation = "git_operation", operation_type = "rebase", repo_path = %
                    worktree.display(), error = % e,
                    "Failed to force cleanup rebase state"
                );
            } else {
                last_error = Some(stderr.to_string());
                continue;
            }
        }
        let after = logging_git::capture_state(worktree).ok();
        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::error!(
            operation = "git_operation", operation_type = "rebase", repo_path = %
            worktree.display(), target, duration_ms, ? before, ? after, result = "error",
            error = % stderr, "Git rebase failed"
        );
        return Err(anyhow::anyhow!("Rebase failed: {stderr}"));
    }
    let after = logging_git::capture_state(worktree).ok();
    let duration_ms = start.elapsed().as_millis() as u64;
    let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
    tracing::error!(
        operation = "git_operation", operation_type = "rebase", repo_path = % worktree
        .display(), target, duration_ms, ? before, ? after, result = "error", error = %
        error_msg, retries = GIT_LOCK_RETRY_COUNT, "Git rebase failed after all retries"
    );
    Err(anyhow::anyhow!("Rebase failed after {} retries: {}", GIT_LOCK_RETRY_COUNT, error_msg))
}
/// Checks if a rebase is currently in progress
pub fn is_rebase_in_progress(worktree: &Path) -> bool {
    let git_dir = match get_git_dir(worktree) {
        Ok(dir) => dir,
        Err(e) => {
            tracing::warn!(
                "Failed to get git directory for {}: {}. Assuming no rebase in progress.",
                worktree.display(),
                e
            );
            return false;
        }
    };
    git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
}
/// Forcefully removes rebase state directories (rebase-merge and rebase-apply).
/// This is used as a fallback when `git rebase --abort` fails because git
/// doesn't recognize the rebase state as valid.
pub fn force_cleanup_rebase_state(worktree: &Path) -> Result<()> {
    let git_dir = get_git_dir(worktree)?;
    let rebase_merge = git_dir.join("rebase-merge");
    let rebase_apply = git_dir.join("rebase-apply");
    let mut removed_any = false;
    if rebase_merge.exists() {
        std::fs::remove_dir_all(&rebase_merge).with_context(|| {
            format!("Failed to remove rebase-merge directory: {}", rebase_merge.display())
        })?;
        tracing::info!("Forcefully removed rebase-merge directory: {}", rebase_merge.display());
        removed_any = true;
    }
    if rebase_apply.exists() {
        std::fs::remove_dir_all(&rebase_apply).with_context(|| {
            format!("Failed to remove rebase-apply directory: {}", rebase_apply.display())
        })?;
        tracing::info!("Forcefully removed rebase-apply directory: {}", rebase_apply.display());
        removed_any = true;
    }
    if !removed_any {
        tracing::debug!("No rebase state directories found to clean up");
    }
    Ok(())
}
/// Checks if worktree is clean (no uncommitted changes and no rebase in
/// progress)
pub fn is_worktree_clean(worktree: &Path) -> Result<bool> {
    Ok(!has_uncommitted_changes(worktree)? && !is_rebase_in_progress(worktree))
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
pub fn abort_rebase(worktree: &Path) -> Result<()> {
    let before = logging_git::capture_state(worktree).ok();
    let start = std::time::Instant::now();
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "rebase_abort", repo_path =
                % worktree.display(), attempt,
                "Retrying rebase abort after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(worktree)
            .arg("rebase")
            .arg("--abort")
            .output()
            .context("Failed to execute git rebase --abort")?;
        if output.status.success() {
            let after = logging_git::capture_state(worktree).ok();
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase_abort", repo_path =
                % worktree.display(), duration_ms, ? before, ? after, result = "success",
                retries = attempt, "Aborted rebase"
            );
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase_abort", repo_path =
                % worktree.display(), attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected during rebase abort, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        if stderr.contains("No rebase in progress") {
            tracing::warn!(
                operation = "git_operation", operation_type = "rebase_abort", repo_path =
                % worktree.display(),
                "Git says no rebase in progress, attempting force cleanup of rebase directories"
            );
            match force_cleanup_rebase_state(worktree) {
                Ok(()) => {
                    let after = logging_git::capture_state(worktree).ok();
                    let duration_ms = start.elapsed().as_millis() as u64;
                    tracing::info!(
                        operation = "git_operation", operation_type = "rebase_abort",
                        repo_path = % worktree.display(), duration_ms, ? before, ? after,
                        result = "success_via_force_cleanup",
                        "Cleaned up orphaned rebase state via force removal"
                    );
                    return Ok(());
                }
                Err(e) => {
                    let after = logging_git::capture_state(worktree).ok();
                    let duration_ms = start.elapsed().as_millis() as u64;
                    tracing::error!(
                        operation = "git_operation", operation_type = "rebase_abort",
                        repo_path = % worktree.display(), duration_ms, ? before, ? after,
                        result = "error", error = % e,
                        "Failed to force cleanup orphaned rebase state"
                    );
                    return Err(e.context("Failed to force cleanup orphaned rebase state"));
                }
            }
        }
        let after = logging_git::capture_state(worktree).ok();
        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::error!(
            operation = "git_operation", operation_type = "rebase_abort", repo_path = %
            worktree.display(), duration_ms, ? before, ? after, result = "error", error =
            % stderr, "Failed to abort rebase"
        );
        return Err(anyhow::anyhow!("Failed to abort rebase: {}", stderr));
    }
    let after = logging_git::capture_state(worktree).ok();
    let duration_ms = start.elapsed().as_millis() as u64;
    let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
    tracing::error!(
        operation = "git_operation", operation_type = "rebase_abort", repo_path = %
        worktree.display(), duration_ms, ? before, ? after, result = "error", error = %
        error_msg, retries = GIT_LOCK_RETRY_COUNT,
        "Failed to abort rebase after all retries"
    );
    Err(anyhow::anyhow!(
        "Failed to abort rebase after {} retries: {}",
        GIT_LOCK_RETRY_COUNT,
        error_msg
    ))
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
                operation = "git_operation", operation_type = "merge_ff", repo_path = %
                repo.display(), branch, duration_ms, ? before, ? after, result =
                "success", "Fast-forward merge succeeded"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation", operation_type = "merge_ff", repo_path = %
                repo.display(), branch, duration_ms, ? before, ? after, result = "error",
                error = % e, "Fast-forward merge failed"
            );
        }
    }
    result
}
pub fn fetch_origin(repo: &Path) -> Result<()> {
    let start = std::time::Instant::now();
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "fetch", repo_path = % repo
                .display(), remote = "origin", attempt,
                "Retrying fetch after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .arg("fetch")
            .arg("origin")
            .output()
            .context("Failed to execute git fetch origin")?;
        if output.status.success() {
            let duration_ms = start.elapsed().as_millis() as u64;
            tracing::debug!(
                operation = "git_operation", operation_type = "fetch", repo_path = % repo
                .display(), remote = "origin", duration_ms, result = "success", retries =
                attempt, "Fetched from origin"
            );
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "fetch", repo_path = % repo
                .display(), remote = "origin", attempt, max_retries =
                GIT_LOCK_RETRY_COUNT, "Git lock file detected, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::error!(
            operation = "git_operation", operation_type = "fetch", repo_path = % repo
            .display(), remote = "origin", duration_ms, result = "error", error = %
            stderr, "Failed to fetch from origin"
        );
        return Err(anyhow::anyhow!("Failed to fetch from origin: {}", stderr));
    }
    let duration_ms = start.elapsed().as_millis() as u64;
    let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
    tracing::error!(
        operation = "git_operation", operation_type = "fetch", repo_path = % repo
        .display(), remote = "origin", duration_ms, result = "error", error = %
        error_msg, retries = GIT_LOCK_RETRY_COUNT,
        "Failed to fetch from origin after all retries"
    );
    Err(anyhow::anyhow!(
        "Failed to fetch from origin after {} retries: {}",
        GIT_LOCK_RETRY_COUNT,
        error_msg
    ))
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
                operation = "git_operation", operation_type = "fetch_local", target_repo
                = % target_repo.display(), source_repo = % source_repo.display(),
                ref_name, duration_ms, result = "success",
                "Fetched from local repository"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "git_operation", operation_type = "fetch_local", target_repo
                = % target_repo.display(), source_repo = % source_repo.display(),
                ref_name, duration_ms, result = "error", error = % e,
                "Failed to fetch from local repository"
            );
        }
    }
    result
}
pub fn checkout_branch(repo: &Path, branch: &str) -> Result<()> {
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "checkout", repo_path = %
                repo.display(), branch, attempt,
                "Retrying checkout after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .arg("checkout")
            .arg(branch)
            .output()
            .context("Failed to execute git checkout")?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "checkout", repo_path = %
                repo.display(), branch, attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected during checkout, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        bail!("Failed to checkout {}: {}", branch, stderr);
    }
    bail!(
        "Failed to checkout {} after {} retries: {}",
        branch,
        GIT_LOCK_RETRY_COUNT,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )
}
pub fn reset_to_ref(repo: &Path, ref_name: &str) -> Result<()> {
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "reset", repo_path = % repo
                .display(), ref_name, attempt, "Retrying reset after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .arg("reset")
            .arg("--hard")
            .arg(ref_name)
            .output()
            .context("Failed to execute git reset")?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "reset", repo_path = % repo
                .display(), ref_name, attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected during reset, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        bail!("Failed to reset to {}: {}", ref_name, stderr);
    }
    bail!(
        "Failed to reset to {} after {} retries: {}",
        ref_name,
        GIT_LOCK_RETRY_COUNT,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )
}
pub fn pull_rebase(worktree: &Path) -> Result<()> {
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "fetch", repo_path = %
                worktree.display(), attempt, "Retrying fetch after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let fetch_output = Command::new("git")
            .arg("-C")
            .arg(worktree)
            .arg("fetch")
            .arg("origin")
            .arg("master")
            .output()
            .context("Failed to execute git fetch")?;
        if fetch_output.status.success() {
            break;
        }
        let stderr = String::from_utf8_lossy(&fetch_output.stderr);
        if is_lock_file_error(&stderr) && attempt < GIT_LOCK_RETRY_COUNT {
            tracing::warn!(
                operation = "git_operation", operation_type = "fetch", repo_path = %
                worktree.display(), attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected during fetch, will retry"
            );
            continue;
        }
        bail!("Failed to fetch from origin: {}", stderr);
    }
    let mut last_error: Option<String> = None;
    for attempt in 0..=GIT_LOCK_RETRY_COUNT {
        if attempt > 0 {
            tracing::debug!(
                operation = "git_operation", operation_type = "pull_rebase", repo_path =
                % worktree.display(), attempt, "Retrying rebase after lock file error"
            );
            thread::sleep(Duration::from_millis(GIT_LOCK_RETRY_DELAY_MS));
        }
        let rebase_output = Command::new("git")
            .arg("-C")
            .arg(worktree)
            .arg("rebase")
            .arg("origin/master")
            .output()
            .context("Failed to execute git rebase")?;
        if rebase_output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&rebase_output.stderr);
        if is_lock_file_error(&stderr) {
            tracing::warn!(
                operation = "git_operation", operation_type = "pull_rebase", repo_path =
                % worktree.display(), attempt, max_retries = GIT_LOCK_RETRY_COUNT,
                "Git lock file detected during rebase, will retry"
            );
            last_error = Some(stderr.to_string());
            continue;
        }
        bail!("Failed to rebase onto origin/master: {}", stderr);
    }
    bail!(
        "Failed to rebase onto origin/master after {} retries: {}",
        GIT_LOCK_RETRY_COUNT,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )
}
/// Checks if a git error message indicates a lock file conflict
fn is_lock_file_error(stderr: &str) -> bool {
    stderr.contains("index.lock': File exists")
        || stderr.contains("Unable to create")
            && (stderr.contains(".lock': File exists") || stderr.contains("lock file exists"))
}
/// Gets the actual git directory path for a worktree.
/// In a worktree, `.git` is a file containing `gitdir: <path>`, not a
/// directory.
fn get_git_dir(worktree: &Path) -> Result<std::path::PathBuf> {
    let git_file = worktree.join(".git");
    if git_file.is_dir() {
        return Ok(git_file);
    }
    if git_file.is_file() {
        let content = std::fs::read_to_string(&git_file).context("Failed to read .git file")?;
        if let Some(gitdir_line) = content.lines().next()
            && let Some(path) = gitdir_line.strip_prefix("gitdir: ")
        {
            let git_dir = if std::path::Path::new(path).is_absolute() {
                std::path::PathBuf::from(path)
            } else {
                worktree.join(path)
            };
            if git_dir.exists() {
                return Ok(git_dir);
            }
        }
    }
    bail!("Could not determine git directory for worktree: {}", worktree.display())
}
