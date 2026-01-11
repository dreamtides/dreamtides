use std::path::Path;

use anyhow::Result;

use crate::logging::types::GitState;

pub fn capture_state(repo: &Path) -> Result<GitState> {
    Ok(GitState {
        branch: crate::git::get_current_branch(repo)?,
        head: crate::git::get_head_commit(repo)
            .map(|s| s.chars().take(8).collect())
            .unwrap_or_else(|_| "unknown".to_string()),
        clean: !crate::git::has_uncommitted_changes(repo)?,
    })
}
