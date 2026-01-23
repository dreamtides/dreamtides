#![allow(dead_code, reason = "Some variants and methods defined for API completeness")]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::auto_mode::auto_logging::TaskErrorContext;

/// Errors that can occur during Claude task file operations.
///
/// These errors are designed to provide detailed context for overseer
/// remediation. Each variant includes the information needed to diagnose and
/// fix the issue.
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Failed to parse task file {path}: {message}")]
    ParseError { path: PathBuf, message: String, raw_content: Option<String> },
    #[error("Missing required field '{field}' in task file {path}")]
    MissingField { path: PathBuf, field: String },
    #[error("Failed to read task file {path}: {message}")]
    ReadError { path: PathBuf, message: String },
    #[error("Failed to write task file {path}: {message}")]
    WriteError { path: PathBuf, message: String },
    #[error("Circular dependency detected: {}", task_ids.join(" -> "))]
    CircularDependency { task_ids: Vec<String> },
    #[error("Task '{task_id}' depends on non-existent task '{missing_id}'")]
    MissingDependency { task_id: String, missing_id: String },
    #[error("Failed to read task directory {path}: {message}")]
    DirectoryError { path: PathBuf, message: String },
    #[error("Claim race lost for task {task_id}: expected owner '{expected}', found '{actual}'")]
    ClaimRaceLost { task_id: String, expected: String, actual: String },
}

/// Legacy error type for backward compatibility with claim_task function.
///
/// This is retained for the specific claim race handling in
/// process_idle_workers where we want to handle race loss differently from
/// other errors.
#[derive(Debug, Error)]
pub enum TaskLifecycleError {
    #[error("Claim race lost for task {task_id}: expected owner '{expected}', found '{actual}'")]
    ClaimRaceLost { task_id: String, expected: String, actual: String },
}

/// Status of a Claude Code task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

/// A task in Claude Code's native task file format.
///
/// Tasks are stored as JSON files in `~/.claude/tasks/<task_list_id>/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeTask {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_form: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Claims a task for a worker with race prevention.
///
/// Sets the task status to `InProgress` and owner to the given worker name,
/// saves atomically, then re-reads the file to verify the owner matches. If
/// verification fails (another process won the race), returns
/// `TaskLifecycleError::ClaimRaceLost`.
pub fn claim_task(
    task: &mut ClaudeTask,
    worker_name: &str,
    task_path: &Path,
) -> Result<(), TaskLifecycleError> {
    task.status = TaskStatus::InProgress;
    task.owner = Some(worker_name.to_string());
    task.save(task_path).map_err(|e| TaskLifecycleError::ClaimRaceLost {
        task_id: task.id.clone(),
        expected: worker_name.to_string(),
        actual: format!("(write error: {})", e),
    })?;
    let reread = ClaudeTask::load(task_path).map_err(|e| TaskLifecycleError::ClaimRaceLost {
        task_id: task.id.clone(),
        expected: worker_name.to_string(),
        actual: format!("(read error: {})", e),
    })?;
    let actual_owner = reread.owner.as_deref().unwrap_or("");
    if actual_owner != worker_name {
        return Err(TaskLifecycleError::ClaimRaceLost {
            task_id: task.id.clone(),
            expected: worker_name.to_string(),
            actual: actual_owner.to_string(),
        });
    }
    info!(task_id = % task.id, worker = % worker_name, "Task claimed successfully");
    Ok(())
}

/// Marks a task as completed after successful accept.
///
/// Loads the task by ID, sets status to `Completed`, clears the owner, and
/// saves atomically.
pub fn complete_task(task_id: &str, task_dir: &Path) -> Result<()> {
    let task_path = task_dir.join(format!("{}.json", task_id));
    let mut task = ClaudeTask::load(&task_path)?;
    task.status = TaskStatus::Completed;
    task.owner = None;
    task.save(&task_path)?;
    info!(task_id = % task_id, "Task marked as completed");
    Ok(())
}

/// Releases a task back to pending status.
///
/// Loads the task by ID, resets status to `Pending`, clears the owner, and
/// saves atomically. Used when a worker fails, crashes, or daemon shuts down.
pub fn release_task(task_id: &str, task_dir: &Path) -> Result<()> {
    let task_path = task_dir.join(format!("{}.json", task_id));
    let mut task = ClaudeTask::load(&task_path)?;
    task.status = TaskStatus::Pending;
    task.owner = None;
    task.save(&task_path)?;
    info!(task_id = % task_id, "Task released back to pending");
    Ok(())
}

impl TaskError {
    /// Returns the file path associated with this error, if any.
    pub fn path(&self) -> Option<&Path> {
        match self {
            TaskError::ParseError { path, .. } => Some(path),
            TaskError::MissingField { path, .. } => Some(path),
            TaskError::ReadError { path, .. } => Some(path),
            TaskError::WriteError { path, .. } => Some(path),
            TaskError::DirectoryError { path, .. } => Some(path),
            TaskError::CircularDependency { .. } => None,
            TaskError::MissingDependency { .. } => None,
            TaskError::ClaimRaceLost { .. } => None,
        }
    }

    /// Returns the task ID associated with this error, if any.
    pub fn task_id(&self) -> Option<&str> {
        match self {
            TaskError::MissingDependency { task_id, .. } => Some(task_id),
            TaskError::ClaimRaceLost { task_id, .. } => Some(task_id),
            _ => None,
        }
    }

    /// Returns the raw content of the file that failed to parse, if available.
    pub fn raw_content(&self) -> Option<&str> {
        match self {
            TaskError::ParseError { raw_content, .. } => raw_content.as_deref(),
            _ => None,
        }
    }

    /// Returns a suggested remediation action for this error.
    pub fn remediation_hint(&self) -> &'static str {
        match self {
            TaskError::ParseError { .. } => {
                "Fix the JSON syntax in the task file. Common issues: missing commas, \
                 unquoted strings, trailing commas, mismatched brackets."
            }
            TaskError::MissingField { .. } => {
                "Add the missing required field to the task JSON file. Required fields: \
                 id, subject, description, status, blocks, blockedBy."
            }
            TaskError::ReadError { .. } => {
                "Check file permissions and ensure the file exists. The task file may have \
                 been deleted or moved."
            }
            TaskError::WriteError { .. } => {
                "Check file permissions and disk space. Ensure the task directory is writable."
            }
            TaskError::CircularDependency { .. } => {
                "Edit the blockedBy fields to remove the circular dependency. Tasks cannot \
                 depend on themselves directly or indirectly."
            }
            TaskError::MissingDependency { .. } => {
                "Either create the missing task file or remove the dependency from the \
                 blockedBy field."
            }
            TaskError::DirectoryError { .. } => {
                "Check that the task directory exists and has correct permissions."
            }
            TaskError::ClaimRaceLost { .. } => {
                "This is a transient error - another process claimed the task first. \
                 The system will automatically retry with another eligible task."
            }
        }
    }
}

impl TaskError {
    /// Converts this error into a TaskErrorContext for persistence.
    pub fn to_error_context(&self) -> super::auto_logging::TaskErrorContext {
        let error_type = match self {
            TaskError::ParseError { .. } => "ParseError",
            TaskError::MissingField { .. } => "MissingField",
            TaskError::ReadError { .. } => "ReadError",
            TaskError::WriteError { .. } => "WriteError",
            TaskError::CircularDependency { .. } => "CircularDependency",
            TaskError::MissingDependency { .. } => "MissingDependency",
            TaskError::DirectoryError { .. } => "DirectoryError",
            TaskError::ClaimRaceLost { .. } => "ClaimRaceLost",
        };
        TaskErrorContext {
            timestamp: super::auto_logging::timestamp_now(),
            error_type: error_type.to_string(),
            error_message: self.to_string(),
            file_path: self.path().map(|p| p.display().to_string()),
            task_id: self.task_id().map(String::from),
            raw_content: self.raw_content().map(String::from),
            remediation_hint: self.remediation_hint().to_string(),
        }
    }
}

impl ClaudeTask {
    /// Loads a task from the given JSON file path.
    pub fn load(path: &Path) -> Result<ClaudeTask> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read task file: {}", path.display()))?;
        let task: ClaudeTask = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse task file: {}", path.display()))?;
        Ok(task)
    }

    /// Loads a task with structured error information for remediation.
    ///
    /// Unlike `load`, this returns a `TaskError` with detailed context that can
    /// be used by the overseer for remediation prompts.
    pub fn load_with_error(path: &Path) -> Result<ClaudeTask, TaskError> {
        let content = fs::read_to_string(path).map_err(|e| TaskError::ReadError {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        serde_json::from_str(&content).map_err(|e| TaskError::ParseError {
            path: path.to_path_buf(),
            message: e.to_string(),
            raw_content: Some(truncate_for_error(&content, 2000)),
        })
    }

    /// Saves the task to the given path with atomic writes.
    ///
    /// Uses temp file + fsync + rename pattern for atomicity.
    #[allow(dead_code, clippy::allow_attributes)]
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create task directory: {}", parent.display())
            })?;
        }
        let json = serde_json::to_string_pretty(self).context("Failed to serialize task")?;
        let temp_filename = format!("{}.tmp.{}", self.id, std::process::id());
        let temp_path = path.with_file_name(&temp_filename);
        let mut file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp task file: {}", temp_path.display()))?;
        file.write_all(json.as_bytes())
            .with_context(|| format!("Failed to write temp task file: {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("Failed to fsync temp task file: {}", temp_path.display()))?;
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "Failed to rename temp file {} to task file {}",
                temp_path.display(),
                path.display()
            )
        })?;
        Ok(())
    }

    /// Returns the task priority from metadata, defaulting to 3.
    ///
    /// Priority values are 0-4 (0 = highest urgency, 4 = lowest).
    pub fn get_priority(&self) -> u8 {
        self.metadata
            .as_ref()
            .and_then(|m| m.get("priority"))
            .and_then(serde_json::Value::as_u64)
            .map(|p| p.min(4) as u8)
            .unwrap_or(3)
    }

    /// Returns the task label from metadata, if present.
    ///
    /// Labels are used for context injection and concurrency optimization.
    pub fn get_label(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.get("label")).and_then(|v| v.as_str())
    }
}

fn truncate_for_error(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncate_at = s.char_indices().take(max_len).last().map(|(i, _)| i).unwrap_or(max_len);
        format!("{}... (truncated)", &s[..truncate_at])
    }
}
