use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::LatticeResult;
use crate::error::error_types::LatticeError;

/// User state stored in `~/.lattice/state.json`.
///
/// Contains state that persists across sessions but is local to the user's
/// machine, not stored in git.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserState {
    /// Last parent directory used with `lat create --interactive`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_create_parent: Option<String>,
}

/// Reads the user state from disk, returning default state if file doesn't
/// exist.
pub fn read_state() -> LatticeResult<UserState> {
    let path = state_file_path()?;

    if !path.exists() {
        debug!(path = %path.display(), "State file does not exist, using defaults");
        return Ok(UserState::default());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| {
        warn!(path = %path.display(), error = %e, "Failed to read state file");
        LatticeError::ReadError { path: path.clone(), reason: e.to_string() }
    })?;

    serde_json::from_str(&content).map_err(|e| {
        warn!(path = %path.display(), error = %e, "Failed to parse state file, using defaults");
        LatticeError::ReadError { path, reason: format!("Invalid JSON: {e}") }
    })
}

/// Writes the user state to disk atomically.
pub fn write_state(state: &UserState) -> LatticeResult<()> {
    let path = state_file_path()?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            warn!(path = %parent.display(), error = %e, "Failed to create state directory");
            LatticeError::WriteError { path: parent.to_path_buf(), reason: e.to_string() }
        })?;
    }

    let content = serde_json::to_string_pretty(state).map_err(|e| LatticeError::WriteError {
        path: path.clone(),
        reason: format!("Failed to serialize state: {e}"),
    })?;

    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, &content).map_err(|e| {
        warn!(path = %temp_path.display(), error = %e, "Failed to write temporary state file");
        LatticeError::WriteError { path: temp_path.clone(), reason: e.to_string() }
    })?;

    std::fs::rename(&temp_path, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        warn!(from = %temp_path.display(), to = %path.display(), error = %e, "Failed to rename state file");
        LatticeError::WriteError { path: path.clone(), reason: e.to_string() }
    })?;

    info!(path = %path.display(), "Wrote state file");
    Ok(())
}

/// Updates the last create parent directory in the user state.
pub fn set_last_create_parent(parent: &str, repo_root: &Path) -> LatticeResult<()> {
    let parent_path = repo_root.join(parent);
    let relative_parent =
        parent_path.strip_prefix(repo_root).unwrap_or(&parent_path).to_string_lossy().to_string();

    let mut state = read_state().unwrap_or_default();
    state.last_create_parent = Some(relative_parent);
    write_state(&state)
}

/// Gets the last create parent directory from the user state.
pub fn get_last_create_parent() -> Option<String> {
    read_state().ok().and_then(|s| s.last_create_parent)
}

/// Returns the user's home directory.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).or({
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("USERPROFILE").map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    })
}

/// Returns the base Lattice directory path (`~/.lattice/`).
fn lattice_base_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".lattice"))
}

/// Returns the path to the user state file.
fn state_file_path() -> LatticeResult<PathBuf> {
    let base = lattice_base_dir().ok_or_else(|| LatticeError::WriteError {
        path: PathBuf::from("~/.lattice"),
        reason: "Could not determine home directory".to_string(),
    })?;
    Ok(base.join("state.json"))
}
