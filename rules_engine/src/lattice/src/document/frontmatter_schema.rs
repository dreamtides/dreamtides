use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::lattice_id::LatticeId;

/// Maximum length for the `name` field in characters.
pub const MAX_NAME_LENGTH: usize = 64;
/// Maximum length for the `description` field in characters.
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
/// Minimum priority value (highest priority).
pub const MIN_PRIORITY: u8 = 0;
/// Maximum priority value (lowest priority / backlog).
pub const MAX_PRIORITY: u8 = 4;
/// Default priority for new tasks.
pub const DEFAULT_PRIORITY: u8 = 2;

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
    /// Tests, docs, refactoring.
    Task,
    /// Dependencies, tooling.
    Chore,
}

/// Parsed and validated frontmatter for a Lattice document.
///
/// All Lattice documents require `lattice_id`, `name`, and `description`.
/// Task documents additionally require `task_type` and `priority`.
///
/// The struct uses serde for YAML serialization with kebab-case field names
/// to match the markdown frontmatter format (e.g., `lattice-id`, `task-type`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Frontmatter {
    // ========================================================================
    // Required Identity Fields (all documents)
    // ========================================================================
    /// Unique document identifier.
    ///
    /// Format: `L` prefix + Base32 document counter + Base32 client ID.
    /// Example: `LVDDTX`.
    pub lattice_id: LatticeId,

    /// Lowercase-hyphenated identifier derived from filename.
    ///
    /// The name is always derived from the document's filename: underscores
    /// become hyphens and the `.md` extension is stripped. For example,
    /// `fix_login_bug.md` → `fix-login-bug`.
    ///
    /// Maximum length: 64 characters.
    pub name: String,

    /// Human-readable summary of the document.
    ///
    /// For tasks, this is the task title shown in `lat show` and list views.
    /// For knowledge base documents, this provides a purpose summary.
    ///
    /// Maximum length: 1024 characters.
    pub description: String,

    // ========================================================================
    // Optional Identity Fields
    // ========================================================================
    /// ID of the parent document.
    ///
    /// Auto-populated by `lat fmt` from the directory's root document (the
    /// document whose filename matches its containing directory name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<LatticeId>,

    // ========================================================================
    // Task Tracking Fields (tasks only)
    // ========================================================================
    /// Type of task: bug, feature, task, or chore.
    ///
    /// The presence of this field distinguishes tasks from knowledge base
    /// documents. Required for tasks; must not be present for KB documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,

    /// Priority level from 0 (highest/critical) to 4 (lowest/backlog).
    ///
    /// - P0: Critical (security, data loss, broken builds)
    /// - P1: High (major features, important bugs)
    /// - P2: Medium (default)
    /// - P3: Low (polish, optimization)
    /// - P4: Backlog (future ideas)
    ///
    /// Required for tasks; must not be present for KB documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// Arbitrary string labels for categorization and filtering.
    ///
    /// Query with `--label` (AND) or `--label-any` (OR).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,

    /// Task IDs that have hard dependencies on this task.
    ///
    /// Tasks listed here cannot start until this task is closed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking: Vec<LatticeId>,

    /// Task IDs this task depends on.
    ///
    /// This task cannot start until all listed tasks are closed.
    /// A task with open blockers is considered "blocked".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<LatticeId>,

    /// Parent task IDs from which this task was discovered.
    ///
    /// Soft link for provenance tracking; does not affect the ready queue.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discovered_from: Vec<LatticeId>,

    // ========================================================================
    // Timestamp Fields
    // ========================================================================
    /// ISO 8601 timestamp when the document was created.
    ///
    /// Auto-set by `lat create`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// ISO 8601 timestamp when the document was last updated.
    ///
    /// Auto-set by `lat update` and `lat fmt` when content changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// ISO 8601 timestamp when the task was closed.
    ///
    /// Auto-set by `lat close`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    // ========================================================================
    // Skill Integration
    // ========================================================================
    /// Whether this document should generate a Claude Skill symlink.
    ///
    /// When true, a symlink is created in `.claude/skills/` pointing to this
    /// document. The `name` and `description` fields must follow Claude's
    /// SKILL.md validation rules.
    #[serde(default, skip_serializing_if = "is_false")]
    pub skill: bool,
}

/// Lenient frontmatter that allows `lattice_id` to be missing.
///
/// Used by `lat track` to parse existing markdown files that have YAML
/// frontmatter but no `lattice-id` field yet. This allows preserving other
/// frontmatter fields while adding the missing ID.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LenientFrontmatter {
    #[serde(default)]
    pub lattice_id: Option<LatticeId>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_id: Option<LatticeId>,
    #[serde(default)]
    pub task_type: Option<TaskType>,
    #[serde(default)]
    pub priority: Option<u8>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub blocking: Vec<LatticeId>,
    #[serde(default)]
    pub blocked_by: Vec<LatticeId>,
    #[serde(default)]
    pub discovered_from: Vec<LatticeId>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub skill: bool,
}

/// Helper for serde `skip_serializing_if` to omit `skill: false`.
fn is_false(value: &bool) -> bool {
    !*value
}

impl Frontmatter {
    /// Returns true if this document is a task (has `task_type` set).
    pub fn is_task(&self) -> bool {
        self.task_type.is_some()
    }

    /// Returns true if this document is a knowledge base document (no
    /// `task_type`).
    pub fn is_knowledge_base(&self) -> bool {
        self.task_type.is_none()
    }

    /// Returns the priority, defaulting to P2 for tasks without explicit
    /// priority.
    ///
    /// Returns `None` for knowledge base documents.
    pub fn effective_priority(&self) -> Option<u8> {
        if self.is_task() { Some(self.priority.unwrap_or(DEFAULT_PRIORITY)) } else { None }
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::Bug => write!(f, "bug"),
            TaskType::Feature => write!(f, "feature"),
            TaskType::Task => write!(f, "task"),
            TaskType::Chore => write!(f, "chore"),
        }
    }
}

impl std::str::FromStr for TaskType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bug" => Ok(TaskType::Bug),
            "feature" => Ok(TaskType::Feature),
            "task" => Ok(TaskType::Task),
            "chore" => Ok(TaskType::Chore),
            _ => Err(format!("invalid task type '{s}': expected one of bug, feature, task, chore")),
        }
    }
}
