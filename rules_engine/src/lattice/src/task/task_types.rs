use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::error_types::LatticeError;

/// Type of a Lattice task.
///
/// Determines the nature of work the task represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    /// Defects and regressions.
    Bug,
    /// User-facing capabilities.
    Feature,
    /// Tests, documentation, refactoring.
    Task,
    /// Dependencies, tooling updates.
    Chore,
}

impl TaskType {
    /// All available task types in display order.
    pub const ALL: [TaskType; 4] =
        [TaskType::Bug, TaskType::Feature, TaskType::Task, TaskType::Chore];

    /// Returns the canonical string representation of this task type.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Bug => "bug",
            TaskType::Feature => "feature",
            TaskType::Task => "task",
            TaskType::Chore => "chore",
        }
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TaskType {
    type Err = LatticeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bug" => Ok(TaskType::Bug),
            "feature" => Ok(TaskType::Feature),
            "task" => Ok(TaskType::Task),
            "chore" => Ok(TaskType::Chore),
            _ => {
                tracing::debug!(value = s, "Invalid task type");
                Err(LatticeError::InvalidArgument {
                    message: format!(
                        "invalid task type '{}': expected one of {}",
                        s,
                        TaskType::ALL.iter().map(TaskType::as_str).collect::<Vec<_>>().join(", ")
                    ),
                })
            }
        }
    }
}
