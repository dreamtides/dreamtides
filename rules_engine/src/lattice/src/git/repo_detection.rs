use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;

/// Types of in-progress git operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InProgressOp {
    /// Interactive or non-interactive rebase in progress.
    Rebase,
    /// Merge in progress (MERGE_HEAD present).
    Merge,
    /// Cherry-pick in progress.
    CherryPick,
    /// Revert in progress.
    Revert,
}

/// Detected repository configuration.
///
/// Captures non-standard git repository states that affect Lattice behavior.
/// Cached in `.lattice/repo_config.json` for fast startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    /// Timestamp when this configuration was detected.
    pub detected_at: String,
    /// Modification time of .git (for cache invalidation).
    pub git_mtime: u64,
    /// True if this is a shallow clone.
    pub is_shallow: bool,
    /// True if this is a partial clone (blobless or treeless).
    pub is_partial: bool,
    /// Filter specification for partial clones (e.g., "blob:none").
    pub partial_filter: Option<String>,
    /// True if sparse checkout is enabled.
    pub is_sparse: bool,
    /// True if this is a worktree (not the main repository).
    pub is_worktree: bool,
    /// Path to the main .git directory (used for client ID sharing).
    pub main_git_dir: PathBuf,
    /// Path to the worktree-specific git directory (for in-progress ops).
    /// Only set when `is_worktree` is true.
    pub worktree_git_dir: Option<PathBuf>,
    /// True if submodules are present.
    pub has_submodules: bool,
    /// True if this is a bare repository (not supported).
    pub is_bare: bool,
    /// In-progress git operation, if any.
    pub in_progress_op: Option<InProgressOp>,
}

impl RepoConfig {
    /// Detects repository configuration for the given root.
    pub fn detect(repo_root: &Path, git: &dyn GitOps) -> Result<Self, LatticeError> {
        let git_path = repo_root.join(".git");
        let git_mtime = get_git_mtime(&git_path);

        info!(repo = %repo_root.display(), "Detecting repository configuration");

        let is_shallow = detect_shallow_clone(repo_root);
        let (is_partial, partial_filter) = detect_partial_clone(git);
        let is_sparse = detect_sparse_checkout(git);
        let (is_worktree, main_git_dir, worktree_git_dir) = detect_worktree(repo_root, git);
        let has_submodules = detect_submodules(repo_root);
        let is_bare = detect_bare_repository(git);
        // Use worktree git dir for in-progress detection if available
        let effective_git_dir = worktree_git_dir.as_ref().unwrap_or(&git_path);
        let in_progress_op = detect_in_progress_op(effective_git_dir);

        let detected_at = Utc::now().to_rfc3339();

        let config = RepoConfig {
            detected_at,
            git_mtime,
            is_shallow,
            is_partial,
            partial_filter,
            is_sparse,
            is_worktree,
            main_git_dir,
            worktree_git_dir,
            has_submodules,
            is_bare,
            in_progress_op,
        };

        log_detection_results(&config);
        Ok(config)
    }

    /// Returns the path to the repo_config.json cache file.
    pub fn cache_path(repo_root: &Path) -> PathBuf {
        repo_root.join(".lattice").join("repo_config.json")
    }

    /// Loads cached configuration if valid.
    ///
    /// Returns `None` if cache is missing, corrupt, or stale.
    pub fn load_cached(repo_root: &Path) -> Result<Option<Self>, LatticeError> {
        let cache_path = Self::cache_path(repo_root);
        if !cache_path.exists() {
            debug!(path = %cache_path.display(), "No cached repo config found");
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path).map_err(|e| LatticeError::ReadError {
            path: cache_path.clone(),
            reason: e.to_string(),
        })?;

        let config: RepoConfig = serde_json::from_str(&content).map_err(|e| {
            LatticeError::ConfigParseError { path: cache_path.clone(), reason: e.to_string() }
        })?;

        // Validate cache freshness
        let git_path = repo_root.join(".git");
        let current_mtime = get_git_mtime(&git_path);

        if config.git_mtime != current_mtime {
            debug!(
                cached_mtime = config.git_mtime,
                current_mtime, "Repo config cache invalidated (git mtime changed)"
            );
            return Ok(None);
        }

        debug!(detected_at = %config.detected_at, "Using cached repo configuration");
        Ok(Some(config))
    }

    /// Saves configuration to cache.
    pub fn save_cache(&self, repo_root: &Path) -> Result<(), LatticeError> {
        let cache_path = Self::cache_path(repo_root);

        // Ensure .lattice directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
                path: parent.to_path_buf(),
                reason: e.to_string(),
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| LatticeError::WriteError {
            path: cache_path.clone(),
            reason: format!("Failed to serialize repo config: {e}"),
        })?;

        fs::write(&cache_path, content).map_err(|e| LatticeError::WriteError {
            path: cache_path.clone(),
            reason: e.to_string(),
        })?;

        debug!(path = %cache_path.display(), "Saved repo configuration to cache");
        Ok(())
    }

    /// Loads configuration, using cache if valid, otherwise detecting fresh.
    pub fn load_or_detect(repo_root: &Path, git: &dyn GitOps) -> Result<Self, LatticeError> {
        if let Some(cached) = Self::load_cached(repo_root)? {
            return Ok(cached);
        }

        let config = Self::detect(repo_root, git)?;
        config.save_cache(repo_root)?;
        Ok(config)
    }
}

/// Gets the modification time of the .git path as seconds since epoch.
fn get_git_mtime(git_path: &Path) -> u64 {
    fs::metadata(git_path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

/// Detects if this is a shallow clone.
fn detect_shallow_clone(repo_root: &Path) -> bool {
    let shallow_file = repo_root.join(".git").join("shallow");
    let is_shallow = shallow_file.exists();
    if is_shallow {
        debug!("Detected shallow clone (.git/shallow exists)");
    }
    is_shallow
}

/// Detects if this is a partial clone (blobless or treeless).
fn detect_partial_clone(git: &dyn GitOps) -> (bool, Option<String>) {
    // Check for promisor remote
    let is_partial = git.config_get("remote.origin.promisor").ok().flatten().is_some();

    let filter = if is_partial {
        git.config_get("remote.origin.partialclonefilter").ok().flatten()
    } else {
        None
    };

    if is_partial {
        debug!(filter = ?filter, "Detected partial clone");
    }

    (is_partial, filter)
}

/// Detects if sparse checkout is enabled.
fn detect_sparse_checkout(git: &dyn GitOps) -> bool {
    let is_sparse = git
        .config_get("core.sparseCheckout")
        .ok()
        .flatten()
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if is_sparse {
        debug!("Detected sparse checkout enabled");
    }
    is_sparse
}

/// Detects if this is a worktree and returns the main and worktree git
/// directories.
///
/// Returns (is_worktree, main_git_dir, worktree_git_dir).
/// For worktrees, main_git_dir is the shared .git directory (for client ID
/// sharing) and worktree_git_dir is the worktree-specific directory (for
/// in-progress ops).
fn detect_worktree(repo_root: &Path, git: &dyn GitOps) -> (bool, PathBuf, Option<PathBuf>) {
    let git_path = repo_root.join(".git");

    // If .git is a file (not directory), this is a worktree
    let is_worktree = git_path.is_file();

    if !is_worktree {
        return (false, git_path, None);
    }

    // Read the gitdir from the .git file
    if let Ok(content) = fs::read_to_string(&git_path)
        && let Some(line) = content.lines().next()
        && let Some(path) = line.strip_prefix("gitdir: ")
    {
        let worktree_git = PathBuf::from(path);
        // Navigate from .git/worktrees/<name> to .git
        if let Some(main_git) = worktree_git.parent().and_then(|p| p.parent()) {
            debug!(
                worktree_git_dir = %worktree_git.display(),
                main_git_dir = %main_git.display(),
                "Detected worktree"
            );
            return (true, main_git.to_path_buf(), Some(worktree_git));
        }
    }

    // Fallback: use git rev-parse --git-common-dir
    let main_git_dir = git
        .rev_parse("--git-common-dir")
        .map(|s| PathBuf::from(s.trim()))
        .unwrap_or_else(|_| git_path.clone());
    debug!(main_git_dir = %main_git_dir.display(), "Detected worktree (via fallback)");
    (true, main_git_dir, None)
}

/// Detects if submodules are present.
fn detect_submodules(repo_root: &Path) -> bool {
    let gitmodules = repo_root.join(".gitmodules");
    let has_submodules = gitmodules.exists();
    if has_submodules {
        debug!("Detected submodules (.gitmodules exists)");
    }
    has_submodules
}

/// Detects if this is a bare repository.
fn detect_bare_repository(git: &dyn GitOps) -> bool {
    let is_bare =
        git.rev_parse("--is-bare-repository").map(|s| s.trim() == "true").unwrap_or(false);

    if is_bare {
        warn!("Detected bare repository - Lattice requires a working directory");
    }
    is_bare
}

/// Detects in-progress git operations in the given git directory.
///
/// For worktrees, pass the worktree-specific git directory (e.g.,
/// `/main/.git/worktrees/wt-name`), not the main .git directory.
fn detect_in_progress_op(git_dir: &Path) -> Option<InProgressOp> {
    // Check for rebase (both interactive and standard)
    if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
        debug!("Detected rebase in progress");
        return Some(InProgressOp::Rebase);
    }

    // Check for merge
    if git_dir.join("MERGE_HEAD").exists() {
        debug!("Detected merge in progress");
        return Some(InProgressOp::Merge);
    }

    // Check for cherry-pick
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        debug!("Detected cherry-pick in progress");
        return Some(InProgressOp::CherryPick);
    }

    // Check for revert
    if git_dir.join("REVERT_HEAD").exists() {
        debug!("Detected revert in progress");
        return Some(InProgressOp::Revert);
    }

    None
}

/// Logs detection results at appropriate levels.
fn log_detection_results(config: &RepoConfig) {
    let mut observations = Vec::new();

    if config.is_shallow {
        observations.push("shallow clone");
    }
    if config.is_partial {
        let filter_desc = config.partial_filter.as_deref().unwrap_or("unknown filter");
        observations.push(filter_desc);
    }
    if config.is_sparse {
        observations.push("sparse checkout");
    }
    if config.is_worktree {
        observations.push("worktree");
    }
    if config.has_submodules {
        observations.push("submodules present");
    }
    if config.is_bare {
        observations.push("BARE REPOSITORY (not supported)");
    }
    if let Some(op) = &config.in_progress_op {
        let op_name = match op {
            InProgressOp::Rebase => "rebase in progress",
            InProgressOp::Merge => "merge in progress",
            InProgressOp::CherryPick => "cherry-pick in progress",
            InProgressOp::Revert => "revert in progress",
        };
        observations.push(op_name);
    }

    if observations.is_empty() {
        info!("Repository configuration: standard (no edge cases detected)");
    } else {
        info!(observations = ?observations, "Repository configuration detected");
    }
}
