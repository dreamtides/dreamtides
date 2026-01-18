use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::LatticeResult;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;

/// Claim data stored in a claim file.
///
/// Contains the timestamp when the task was claimed and the worktree path
/// from which it was claimed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClaimData {
    /// ISO 8601 timestamp when the claim was created.
    pub claimed_at: DateTime<Utc>,
    /// Path to the worktree from which this task was claimed.
    pub work_path: PathBuf,
}

/// Result of attempting to write a claim exclusively.
pub enum WriteClaimResult {
    /// Successfully created the claim file.
    Created,
    /// The claim file already exists.
    AlreadyExists,
}

/// Returns the claims directory path for the given repository.
///
/// Path format: `~/.lattice/claims/<repo-hash>/`
/// where `<repo-hash>` is the first 8 characters of the SHA-256 hash
/// of the canonical repository root path.
pub fn claim_dir_path(repo_root: &Path) -> LatticeResult<PathBuf> {
    let base = lattice_base_dir().ok_or_else(|| LatticeError::WriteError {
        path: PathBuf::from("~/.lattice"),
        reason: "Could not determine home directory".to_string(),
    })?;
    let repo_hash = compute_repo_hash(repo_root)?;
    Ok(base.join("claims").join(repo_hash))
}

/// Returns the claim file path for a specific task in a repository.
///
/// Path format: `~/.lattice/claims/<repo-hash>/<lattice-id>.json`
pub fn claim_file_path(repo_root: &Path, id: &LatticeId) -> LatticeResult<PathBuf> {
    let dir = claim_dir_path(repo_root)?;
    Ok(dir.join(format!("{}.json", id.as_str())))
}

/// Writes a claim to the specified path atomically.
///
/// Creates parent directories if they don't exist. Uses write-then-rename
/// for atomic file creation on POSIX systems.
pub fn write_claim(path: &Path, claim: &ClaimData) -> LatticeResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            warn!(path = %parent.display(), error = %e, "Failed to create claims directory");
            LatticeError::WriteError { path: parent.to_path_buf(), reason: e.to_string() }
        })?;
        debug!(path = %parent.display(), "Ensured claims directory exists");
    }

    let content = serde_json::to_string_pretty(claim).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize claim: {e}"),
    })?;

    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, &content).map_err(|e| {
        warn!(path = %temp_path.display(), error = %e, "Failed to write temporary claim file");
        LatticeError::WriteError { path: temp_path.clone(), reason: e.to_string() }
    })?;

    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        warn!(from = %temp_path.display(), to = %path.display(), error = %e, "Failed to rename claim file");
        LatticeError::WriteError { path: path.to_path_buf(), reason: e.to_string() }
    })?;

    info!(path = %path.display(), "Wrote claim file");
    Ok(())
}

/// Writes a claim to the specified path atomically, failing if the file exists.
///
/// Creates parent directories if they don't exist. Uses `create_new` for
/// atomic exclusive creation to avoid TOCTOU races.
///
/// Returns `AlreadyExists` if the file already exists.
pub fn write_claim_exclusive(path: &Path, claim: &ClaimData) -> LatticeResult<WriteClaimResult> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            warn!(path = %parent.display(), error = %e, "Failed to create claims directory");
            LatticeError::WriteError { path: parent.to_path_buf(), reason: e.to_string() }
        })?;
    }

    let content = serde_json::to_string_pretty(claim).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize claim: {e}"),
    })?;

    // Use create_new for atomic exclusive creation
    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            debug!(path = %path.display(), "Claim file already exists");
            return Ok(WriteClaimResult::AlreadyExists);
        }
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Failed to create claim file");
            return Err(LatticeError::WriteError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            });
        }
    };

    file.write_all(content.as_bytes()).map_err(|e| {
        warn!(path = %path.display(), error = %e, "Failed to write claim file");
        let _ = std::fs::remove_file(path);
        LatticeError::WriteError { path: path.to_path_buf(), reason: e.to_string() }
    })?;

    info!(path = %path.display(), "Wrote claim file exclusively");
    Ok(WriteClaimResult::Created)
}

/// Reads a claim from the specified path if it exists.
///
/// Returns `Ok(None)` if the file doesn't exist.
/// Returns an error for other I/O or parsing failures.
pub fn read_claim(path: &Path) -> LatticeResult<Option<ClaimData>> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            debug!(path = %path.display(), "Claim file does not exist");
            return Ok(None);
        }
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Failed to read claim file");
            return Err(LatticeError::ReadError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            });
        }
    };

    let claim: ClaimData = serde_json::from_str(&content).map_err(|e| {
        warn!(path = %path.display(), error = %e, "Failed to parse claim file");
        LatticeError::ReadError {
            path: path.to_path_buf(),
            reason: format!("Invalid claim file format: {e}"),
        }
    })?;

    debug!(path = %path.display(), "Read claim file");
    Ok(Some(claim))
}

/// Deletes a claim file at the specified path.
///
/// Returns success if the file was deleted or didn't exist.
/// Returns an error only for other I/O failures (e.g., permission denied).
pub fn delete_claim(path: &Path) -> LatticeResult<()> {
    match std::fs::remove_file(path) {
        Ok(()) => {
            info!(path = %path.display(), "Deleted claim file");
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            debug!(path = %path.display(), "Claim file already deleted");
            Ok(())
        }
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Failed to delete claim file");
            Err(LatticeError::WriteError { path: path.to_path_buf(), reason: e.to_string() })
        }
    }
}

impl ClaimData {
    /// Creates a new claim with the current timestamp and the given work path.
    pub fn new(work_path: PathBuf) -> Self {
        Self { claimed_at: Utc::now(), work_path }
    }
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
///
/// Returns `None` if the home directory cannot be determined.
fn lattice_base_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".lattice"))
}

/// Computes the repository hash: first 8 characters of SHA-256 of canonical
/// path.
fn compute_repo_hash(repo_root: &Path) -> LatticeResult<String> {
    let canonical = repo_root.canonicalize().map_err(|e| LatticeError::ReadError {
        path: repo_root.to_path_buf(),
        reason: format!("Failed to canonicalize repository path: {e}"),
    })?;
    let path_str = canonical.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hasher.finalize();
    // Take first 4 bytes and format as 8 hex characters
    Ok(format!("{:02x}{:02x}{:02x}{:02x}", hash[0], hash[1], hash[2], hash[3]))
}
