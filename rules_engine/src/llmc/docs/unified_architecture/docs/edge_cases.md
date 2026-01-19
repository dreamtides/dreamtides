---
lattice-id: LB2WQN
parent-id: LBUWQN
name: edge-cases
description: |-
  Git state edge cases, state inconsistencies, and failure recovery matrix for
  unified architecture.
created-at: 2026-01-19T05:00:00.000000Z
updated-at: 2026-01-19T05:00:00.000000Z
---

# Edge Cases and Failure Modes

## Git State Edge Cases

| Scenario | Detection | Handling |
|----------|-----------|----------|
| Lock file exists | Check `index.lock` | Retry with backoff, then fail with message |
| Rebase in progress | Check `.git/rebase-merge/` | Block operation, instruct user to resolve |
| Merge in progress | Check `MERGE_HEAD` | Block operation, instruct user to resolve |
| Detached HEAD in worktree | `git symbolic-ref HEAD` fails | Reattach to branch or reset |
| Worktree on wrong branch | Compare branch name | Reset to correct branch |
| Orphaned worktree (no branch) | Branch doesn't exist | Remove and recreate |
| Corrupted worktree | Various git errors | Remove and recreate |
| Main repo dirty | `has_uncommitted_changes()` | Block accept, inform user |
| Main repo not on master | `get_current_branch()` | Block accept, inform user |

## State Inconsistencies

| Scenario | Detection | Handling |
|----------|-----------|----------|
| Worker in state, no worktree | Path doesn't exist | Mark offline, recreate on start |
| Worktree exists, not in state | Scan `.llmc-worktrees/` | Add to state or remove worktree |
| Branch exists, no worker | List `llmc/*` branches | Delete orphaned branch |
| Worker online, no TMUX session | `session_exists()` returns false | Mark offline |
| Session exists, worker offline | Session check in patrol | Mark online or kill session |
| Commit SHA mismatch | Compare stored vs actual | Update state |

## Failure Recovery Matrix

| Failure Point | Symptoms | Automatic Recovery | Manual Recovery |
|---------------|----------|-------------------|-----------------|
| Init interrupted | Partial metadata dir | `llmc init --force` | Remove ~/llmc, reinit |
| Add interrupted | Orphaned branch/worktree | `llmc doctor --repair` | Manual cleanup |
| Accept during rebase | Worker stuck in rebasing | Patrol detects, prompts | `llmc reset <worker>` |
| Accept during squash | Partially squashed | `llmc reset <worker>` | Manual git operations |
| Accept during merge | Merge conflict | Should not happen (ff-only) | `git merge --abort` |
| Nuke interrupted | Partial cleanup | `llmc doctor --repair` | Manual cleanup |
| Reset interrupted | Inconsistent state | `llmc doctor --repair` | `llmc nuke` + `llmc add` |

## Concurrent Operation Safety

```rust
/// Operations that must not run concurrently
const EXCLUSIVE_OPERATIONS: &[&str] = &[
    "accept",
    "reset",
    "nuke",
    "add",
    "init",
];

/// Acquire exclusive lock for dangerous operations
pub fn acquire_exclusive_lock(operation: &str) -> Result<ExclusiveLock> {
    let lock_path = config::get_llmc_root().join(".llmc-exclusive.lock");

    // Try to acquire lock with timeout
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
        match ExclusiveLock::try_acquire(&lock_path, operation) {
            Ok(lock) => return Ok(lock),
            Err(_) if start.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}
```
