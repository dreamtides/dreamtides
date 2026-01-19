---
lattice-id: LBZWQN
parent-id: LBUWQN
name: git-operation-safety
description: |-
  Pre-operation checks, atomic operations, and worktree safety for unified
  architecture.
created-at: 2026-01-19T05:00:00.000000Z
updated-at: 2026-01-19T05:00:00.000000Z
---

# Git Operation Safety

## Pre-Operation Checks

Before any destructive git operation, verify:

```rust
/// Comprehensive safety check before modifying git state
pub fn verify_safe_to_modify(repo: &Path, operation: &str) -> Result<()> {
    // 1. Verify repo exists and is a git repo
    if !repo.join(".git").exists() && !repo.join("HEAD").exists() {
        bail!("Not a git repository: {}", repo.display());
    }

    // 2. Check for lock files
    let git_dir = get_git_dir(repo)?;
    let lock_files = ["index.lock", "HEAD.lock", "config.lock"];
    for lock in &lock_files {
        let lock_path = git_dir.join(lock);
        if lock_path.exists() {
            bail!(
                "Git lock file exists: {}\n\
                 Another git operation may be in progress.\n\
                 If not, remove the lock file manually.",
                lock_path.display()
            );
        }
    }

    // 3. Check for in-progress operations
    if is_rebase_in_progress(repo) {
        bail!("Rebase in progress - cannot perform {}", operation);
    }
    if is_merge_in_progress(repo)? {
        bail!("Merge in progress - cannot perform {}", operation);
    }
    if is_cherry_pick_in_progress(repo)? {
        bail!("Cherry-pick in progress - cannot perform {}", operation);
    }

    Ok(())
}
```

## Atomic Operations

All state-modifying operations should be atomic:

```rust
/// Performs an operation with automatic rollback on failure
pub fn with_rollback<F, R>(
    repo: &Path,
    description: &str,
    operation: F,
) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    // Save current HEAD for rollback
    let original_head = get_head_commit(repo)?;
    let original_branch = get_current_branch(repo).ok();

    match operation() {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!(
                "Operation '{}' failed, attempting rollback to {}",
                description, original_head
            );

            // Attempt rollback
            if let Err(rollback_err) = reset_to_ref(repo, &original_head) {
                tracing::error!(
                    "Rollback failed: {}. Manual intervention required.",
                    rollback_err
                );
            }

            Err(e.context(format!("Operation '{}' failed", description)))
        }
    }
}
```

## Worktree Safety

```rust
/// Safely removes a worktree with verification
pub fn safe_remove_worktree(repo: &Path, worktree: &Path) -> Result<()> {
    // 1. Verify worktree exists
    if !worktree.exists() {
        tracing::warn!("Worktree doesn't exist, skipping: {}", worktree.display());
        return Ok(());
    }

    // 2. Verify it's actually a worktree (has .git file, not directory)
    let git_file = worktree.join(".git");
    if git_file.is_dir() {
        bail!(
            "Path appears to be a full git repo, not a worktree: {}\n\
             Refusing to remove to prevent data loss.",
            worktree.display()
        );
    }

    // 3. Check for uncommitted changes
    if has_uncommitted_changes(worktree)? {
        tracing::warn!(
            "Worktree has uncommitted changes that will be lost: {}",
            worktree.display()
        );
    }

    // 4. Remove with force flag
    remove_worktree(repo, worktree, true)
}
```
