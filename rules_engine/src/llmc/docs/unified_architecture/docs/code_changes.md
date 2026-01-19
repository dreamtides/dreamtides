---
lattice-id: LBYWQN
name: code-changes
description: File-by-file implementation guide for unified LLMC architecture migration.
parent-id: LBUWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-19T05:08:18.088062Z
---

# Code Changes by File

## Summary Table

| File | Change Type | Complexity | Description |
|------|-------------|------------|-------------|
| `config.rs` | Modify | Medium | Add `worktree_dir()`, `metadata_dir()`, `git_repo()` methods |
| `commands/init.rs` | Rewrite | High | Remove clone, create metadata dir only, update gitignore |
| `commands/add.rs` | Modify | Medium | Use `config.git_repo()` and `config.worktree_dir()` |
| `commands/accept.rs` | Rewrite | High | Simplify flow, remove fetch-into-source |
| `commands/start.rs` | Modify | Low | Update path references |
| `commands/reset.rs` | Modify | Low | Update path references |
| `commands/nuke.rs` | Modify | Low | Update path references |
| `commands/rebase.rs` | Modify | Medium | Change `origin/master` to `master` |
| `commands/doctor.rs` | Modify | Medium | Update health checks for new architecture |
| `commands/review.rs` | Modify | Low | Update path references |
| `patrol.rs` | Modify | Medium | Change `origin/master` to `master` |
| `git.rs` | Modify | Medium | Add safety checks, update fetch logic |
| `state.rs` | No change | - | Format unchanged |
| `worker.rs` | No change | - | Logic unchanged |

## Global Search-Replace Pattern

In all files, apply these changes:

| Old Pattern | New Pattern | Notes |
|-------------|-------------|-------|
| `origin/master` | `master` | All git ref comparisons |
| `git::fetch_origin(&llmc_root)?` | Remove or replace | No longer needed |
| `config::get_llmc_root()` (for git ops) | `config.source_repo()` | Git operations |
| `llmc_root.join(".worktrees")` | `config.worktree_dir()` | Worktree paths |

## Detailed Changes

### config.rs

**Add to `RepoConfig` struct:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub source: String,
    #[serde(default)]
    pub worktree_dir: Option<String>,
    #[serde(default)]
    pub metadata_dir: Option<String>,
}
```

**Add helper methods to `Config`:**
```rust
impl Config {
    pub fn worktree_dir(&self) -> PathBuf {
        self.repo.worktree_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.source_repo().join(".llmc-worktrees"))
    }

    pub fn source_repo(&self) -> PathBuf {
        // Expand ~ in path
        let path = &self.repo.source;
        if path.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }
}
```

### git.rs - New Functions

**Add merge-in-progress check:**
```rust
pub fn is_merge_in_progress(repo: &Path) -> Result<bool> {
    let git_dir = get_git_dir(repo)?;
    Ok(git_dir.join("MERGE_HEAD").exists())
}
```

**Add cherry-pick-in-progress check:**
```rust
pub fn is_cherry_pick_in_progress(repo: &Path) -> Result<bool> {
    let git_dir = get_git_dir(repo)?;
    Ok(git_dir.join("CHERRY_PICK_HEAD").exists())
}
```

**Modify all `origin/master` references:**

The following functions need `origin/master` changed to `master`:

- `pull_rebase()` - remove fetch, just rebase onto master
- `rebase_onto()` - already parameterized, callers change
- `has_commits_ahead_of()` - callers pass `master` instead of `origin/master`
