use std::path::Path;

use tracing::{info, warn};

use crate::error::error_types::LatticeError;
use crate::git::repo_detection::{InProgressOp, RepoConfig};

/// Validates that the repository is in a supported configuration.
///
/// Returns an error if the repository is in an unsupported state (e.g., bare).
/// Logs warnings for degraded states that are supported but may have
/// limitations.
pub fn validate_repo_state(config: &RepoConfig) -> Result<(), LatticeError> {
    if config.is_bare {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Lattice requires a working directory. This appears to be a bare repository."
                .to_string(),
        });
    }

    // Log warnings for degraded states
    if config.is_shallow {
        warn!(
            "This repository uses a shallow clone. \
             Incremental reconciliation may fall back to full rebuilds."
        );
    }

    if config.is_partial {
        let filter = config.partial_filter.as_deref().unwrap_or("unknown");
        if filter.contains("tree:0") || filter == "tree:0" {
            warn!(
                filter = filter,
                "This repository uses a treeless partial clone. \
                 Some operations may be slow or trigger network fetches."
            );
        } else {
            info!(
                filter = filter,
                "This repository uses a partial clone. \
                 Reading documents may trigger network fetches."
            );
        }
    }

    if config.is_sparse {
        info!(
            "Sparse checkout detected. \
             Documents outside your sparse patterns won't be indexed."
        );
    }

    if let Some(op) = &config.in_progress_op {
        warn!(
            operation = ?op,
            "Git operation in progress. Files with unresolved conflicts will be skipped."
        );
    }

    Ok(())
}

/// Generates a user-friendly message about the repository state.
///
/// Includes remediation suggestions for non-standard configurations.
pub fn repo_state_message(config: &RepoConfig) -> Option<String> {
    let mut messages = Vec::new();

    if config.is_shallow {
        messages.push(
            "Warning: This repository uses a shallow clone.\n\
             To enable full functionality, run: git fetch --unshallow"
                .to_string(),
        );
    }

    if config.is_partial {
        let filter = config.partial_filter.as_deref().unwrap_or("unknown filter");
        if filter.contains("tree:0") {
            messages.push(format!(
                "Warning: This repository uses a treeless partial clone ({filter}).\n\
                 For better performance, consider using: git clone --filter=blob:none"
            ));
        }
    }

    if config.is_sparse {
        messages.push(
            "Note: Sparse checkout is enabled.\n\
             Documents outside your sparse patterns won't be indexed.\n\
             To include more: git sparse-checkout add <pattern>"
                .to_string(),
        );
    }

    if let Some(op) = &config.in_progress_op {
        let op_name = match op {
            InProgressOp::Rebase => "rebase",
            InProgressOp::Merge => "merge",
            InProgressOp::CherryPick => "cherry-pick",
            InProgressOp::Revert => "revert",
        };
        messages.push(format!(
            "Warning: A git {op_name} is in progress.\n\
             Files with unresolved conflicts will be skipped.\n\
             Run 'git status' to see conflict state."
        ));
    }

    if config.has_submodules {
        messages.push(
            "Note: Submodules detected.\n\
             Each submodule has its own Lattice namespace.\n\
             Run 'lat' commands within submodule directories to manage their documents."
                .to_string(),
        );
    }

    if messages.is_empty() { None } else { Some(messages.join("\n\n")) }
}

/// Checks if incremental reconciliation should be attempted.
///
/// Returns `false` for configurations where incremental reconciliation is
/// unreliable or unavailable.
pub fn should_attempt_incremental_reconciliation(config: &RepoConfig) -> bool {
    // Shallow clones may not have the commit history needed for diff
    // Still attempt it, but be prepared for fallback
    if config.is_shallow {
        info!("Shallow clone detected, incremental reconciliation may fall back to full rebuild");
    }

    // Bare repos don't have working directories
    if config.is_bare {
        return false;
    }

    true
}

/// Returns the effective git directory for in-progress operation checks.
///
/// For worktrees, this returns the worktree-specific directory where operation
/// state files (MERGE_HEAD, rebase-merge, etc.) live.
pub fn effective_git_dir(repo_root: &Path, config: &RepoConfig) -> std::path::PathBuf {
    if config.is_worktree {
        // For worktrees, operation state is in the worktree-specific git directory
        config.worktree_git_dir.clone().unwrap_or_else(|| repo_root.join(".git"))
    } else {
        repo_root.join(".git")
    }
}

/// Generates guidance for sparse checkout situations.
///
/// Returns a message if a requested document is outside the sparse checkout.
pub fn sparse_checkout_guidance(document_path: &Path, _config: &RepoConfig) -> String {
    format!(
        "Document {} is outside your sparse checkout.\n\
         Run: git sparse-checkout add {}\n\
         Then retry your command.",
        document_path.display(),
        document_path.display()
    )
}
