---
lattice-id: LRPWQN
name: configuration-changes
description: New configuration schema and resolution logic for unified LLMC architecture.
parent-id: LRMWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:24.854913Z
---

# Configuration Changes

## New Config Schema

```toml
[defaults]
model = "opus"
skip_permissions = true
patrol_interval_secs = 60
sound_on_review = true

[repo]
source = "~/Documents/GoogleDrive/dreamtides"

# NEW: Optional override for worktree location
# worktree_dir = "~/Documents/GoogleDrive/dreamtides/.llmc-worktrees"

# NEW: Optional override for metadata location
# metadata_dir = "~/llmc"

[workers.adam]
model = "opus"
```

## Config Resolution Logic

```rust
impl Config {
    /// Returns the directory containing worker worktrees
    pub fn worktree_dir(&self) -> PathBuf {
        self.repo.worktree_dir
            .clone()
            .unwrap_or_else(|| {
                PathBuf::from(&self.repo.source).join(".llmc-worktrees")
            })
    }

    /// Returns the directory containing LLMC metadata (config, state, logs)
    pub fn metadata_dir(&self) -> PathBuf {
        self.repo.metadata_dir
            .clone()
            .unwrap_or_else(config::get_llmc_root)
    }

    /// Returns the path to the git repository for all git operations
    pub fn git_repo(&self) -> PathBuf {
        PathBuf::from(&self.repo.source)
    }
}
```
