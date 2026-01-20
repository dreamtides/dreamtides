use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::error::error_types::LatticeError;

/// Path segment that indicates a task is in a closed directory.
const CLOSED_DIR_SEGMENT: &str = "/.closed/";
/// Directory name for closed tasks.
pub const CLOSED_DIR_NAME: &str = ".closed";

/// Checks if a path represents a closed task by containing the `/.closed/`
/// segment.
///
/// This is a fast path check that examines the path string directly without
/// filesystem access.
///
/// # Examples
///
/// ```ignore
/// assert!(is_in_closed("auth/tasks/.closed/fix_bug.md"));
/// assert!(!is_in_closed("auth/tasks/fix_bug.md"));
/// ```
pub fn is_in_closed(path: &str) -> bool {
    path.contains(CLOSED_DIR_SEGMENT)
}

/// Computes the closed path for a task.
///
/// Given a task path like `auth/tasks/foo.md`, returns the corresponding
/// closed path `auth/tasks/.closed/foo.md`.
///
/// # Arguments
///
/// * `task_path` - Path to the open task document
///
/// # Returns
///
/// The destination path in the `.closed/` directory.
///
/// # Errors
///
/// Returns `LatticeError::InvalidPath` if:
/// - The path is already in a `.closed/` directory
/// - The path contains nested `.closed/` directories
/// - The path has no parent directory
pub fn closed_path_for(task_path: &Path) -> Result<PathBuf, LatticeError> {
    let path_str = task_path.to_string_lossy();

    if is_in_closed(&path_str) {
        return Err(LatticeError::InvalidPath {
            path: task_path.to_path_buf(),
            reason: "Task is already in a .closed/ directory".to_string(),
        });
    }

    if has_nested_closed(&path_str) {
        return Err(LatticeError::InvalidPath {
            path: task_path.to_path_buf(),
            reason: "Path contains nested .closed/ directories".to_string(),
        });
    }

    let parent = task_path.parent().ok_or_else(|| LatticeError::InvalidPath {
        path: task_path.to_path_buf(),
        reason: "Task path has no parent directory".to_string(),
    })?;

    let file_name = task_path.file_name().ok_or_else(|| LatticeError::InvalidPath {
        path: task_path.to_path_buf(),
        reason: "Task path has no filename".to_string(),
    })?;

    let closed_dir = parent.join(CLOSED_DIR_NAME);
    let result = closed_dir.join(file_name);

    debug!(
        task_path = %task_path.display(),
        closed_path = %result.display(),
        "Computed closed path for task"
    );

    Ok(result)
}

/// Computes the original path for a closed task.
///
/// Given a closed task path like `auth/tasks/.closed/foo.md`, returns the
/// original path `auth/tasks/foo.md`.
///
/// # Arguments
///
/// * `closed_path` - Path to the closed task document
///
/// # Returns
///
/// The original path outside the `.closed/` directory.
///
/// # Errors
///
/// Returns `LatticeError::InvalidPath` if:
/// - The path is not in a `.closed/` directory
/// - The path structure is malformed
pub fn unclosed_path_for(closed_path: &Path) -> Result<PathBuf, LatticeError> {
    let path_str = closed_path.to_string_lossy();

    if !is_in_closed(&path_str) {
        return Err(LatticeError::InvalidPath {
            path: closed_path.to_path_buf(),
            reason: "Task is not in a .closed/ directory".to_string(),
        });
    }

    let file_name = closed_path.file_name().ok_or_else(|| LatticeError::InvalidPath {
        path: closed_path.to_path_buf(),
        reason: "Closed path has no filename".to_string(),
    })?;

    let closed_dir = closed_path.parent().ok_or_else(|| LatticeError::InvalidPath {
        path: closed_path.to_path_buf(),
        reason: "Closed path has no parent directory".to_string(),
    })?;

    let closed_dir_name = closed_dir.file_name().ok_or_else(|| LatticeError::InvalidPath {
        path: closed_path.to_path_buf(),
        reason: "Cannot determine .closed directory name".to_string(),
    })?;

    if closed_dir_name != CLOSED_DIR_NAME {
        return Err(LatticeError::InvalidPath {
            path: closed_path.to_path_buf(),
            reason: format!(
                "Expected .closed directory but found '{}'",
                closed_dir_name.to_string_lossy()
            ),
        });
    }

    let tasks_dir = closed_dir.parent().ok_or_else(|| LatticeError::InvalidPath {
        path: closed_path.to_path_buf(),
        reason: ".closed directory has no parent".to_string(),
    })?;

    let result = tasks_dir.join(file_name);

    debug!(
        closed_path = %closed_path.display(),
        unclosed_path = %result.display(),
        "Computed unclosed path for task"
    );

    Ok(result)
}

/// Ensures the `.closed/` directory exists for a given parent path.
///
/// Creates the directory if it does not exist. The parent path should be
/// the directory containing tasks (typically a `tasks/` directory).
///
/// # Arguments
///
/// * `parent_path` - Path to the parent directory (e.g., `auth/tasks/`)
/// * `repo_root` - Repository root for computing absolute paths
///
/// # Returns
///
/// The path to the `.closed/` directory.
///
/// # Errors
///
/// Returns `LatticeError::WriteError` if directory creation fails.
pub fn ensure_closed_dir(parent_path: &Path, repo_root: &Path) -> Result<PathBuf, LatticeError> {
    let closed_dir = parent_path.join(CLOSED_DIR_NAME);
    let absolute_closed_dir = repo_root.join(&closed_dir);

    if !absolute_closed_dir.exists() {
        info!(
            closed_dir = %closed_dir.display(),
            "Creating .closed directory"
        );
        fs::create_dir_all(&absolute_closed_dir).map_err(|e| LatticeError::WriteError {
            path: closed_dir.clone(),
            reason: format!("Failed to create .closed directory: {}", e),
        })?;
    } else {
        debug!(
            closed_dir = %closed_dir.display(),
            ".closed directory already exists"
        );
    }

    Ok(closed_dir)
}

/// Validates that a path follows the expected closed directory structure.
///
/// # Arguments
///
/// * `path` - Path to validate
///
/// # Errors
///
/// Returns `LatticeError::InvalidStructure` if the path contains nested
/// `.closed/` directories.
pub fn validate_closed_path_structure(path: &Path) -> Result<(), LatticeError> {
    let path_str = path.to_string_lossy();

    if has_nested_closed(&path_str) {
        return Err(LatticeError::InvalidStructure {
            path: path.to_path_buf(),
            reason: "Path contains nested .closed/ directories".to_string(),
        });
    }

    Ok(())
}

/// Checks if the path contains multiple `.closed/` segments.
fn has_nested_closed(path: &str) -> bool {
    path.matches(CLOSED_DIR_SEGMENT).count() > 1
}
