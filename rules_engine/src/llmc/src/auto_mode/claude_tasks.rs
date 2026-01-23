use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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

impl ClaudeTask {
    /// Loads a task from the given JSON file path.
    pub fn load(path: &Path) -> Result<ClaudeTask> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read task file: {}", path.display()))?;
        let task: ClaudeTask = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse task file: {}", path.display()))?;
        Ok(task)
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
