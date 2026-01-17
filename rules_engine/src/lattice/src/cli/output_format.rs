use std::fmt::{self, Write as _};

use serde::Serialize;

use crate::cli::color_theme;
use crate::document::frontmatter_schema::TaskType;
use crate::id::lattice_id::LatticeId;
/// Unicode box-drawing characters for tree display.
pub mod tree_chars {
    /// Vertical line: │
    pub const VERTICAL: &str = "│";
    /// Horizontal line: ─
    pub const HORIZONTAL: &str = "─";
    /// Branch continuation: ├
    pub const BRANCH: &str = "├";
    /// Last branch: └
    pub const LAST_BRANCH: &str = "└";
    /// Horizontal with vertical down: ┬
    pub const HORIZONTAL_DOWN: &str = "┬";
    /// Cross: ┼
    pub const CROSS: &str = "┼";
}

/// Maximum display width for descriptions in text output.
const MAX_DESCRIPTION_DISPLAY_WIDTH: usize = 80;
/// Default truncation width for names in compact displays.
const DEFAULT_NAME_TRUNCATE_WIDTH: usize = 32;

/// Output format selection for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable plain text output.
    #[default]
    Text,
    /// Structured JSON output for programmatic access.
    Json,
    /// Visual output with colors and box-drawing characters.
    Pretty,
}

/// Task state computed from filesystem location and dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// Task is in `.closed/` directory.
    Closed,
    /// Task has open (non-closed) entries in `blocked-by` field.
    Blocked,
    /// Task exists outside `.closed/` with no open blockers.
    Open,
}

/// Type indicator for document references.
///
/// Displayed in brackets at the end of a document reference.
/// Tasks show priority (e.g., P0, P1), closed tasks append "/closed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeIndicator {
    /// Task with priority, state, and type (e.g., "P0" or "P1/closed").
    Task { priority: u8, closed: bool, task_type: TaskType },
    /// Knowledge base document.
    KnowledgeBase,
}

/// A reference to a document for display in various output formats.
///
/// Provides consistent formatting across all CLI commands using the pattern
/// `<id>: <name> - <description> [<type-indicator>]`.
#[derive(Debug, Clone)]
pub struct DocumentRef {
    /// The Lattice ID.
    pub id: LatticeId,
    /// Lowercase-hyphenated identifier derived from filename.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Type indicator showing priority/state or "doc".
    pub type_indicator: TypeIndicator,
}

/// JSON representation of a document reference.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentRefJson {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<TaskState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
}

/// A node in a tree structure for pretty printing.
#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    /// The data for this node.
    pub data: T,
    /// Child nodes.
    pub children: Vec<TreeNode<T>>,
}

/// Formats a tree structure for display.
pub struct TreeFormatter;

/// Truncates a string to the specified width, adding ellipsis if needed.
pub fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        ".".repeat(max_width)
    } else {
        let truncated: String = s.chars().take(max_width - 3).collect();
        format!("{truncated}...")
    }
}

/// Truncates a description to the default display width.
pub fn truncate_description(description: &str) -> String {
    truncate_with_ellipsis(description, MAX_DESCRIPTION_DISPLAY_WIDTH)
}

/// Truncates a name to the default compact width.
pub fn truncate_name(name: &str) -> String {
    truncate_with_ellipsis(name, DEFAULT_NAME_TRUNCATE_WIDTH)
}

/// Formats a priority value for display.
pub fn format_priority(priority: u8) -> String {
    format!("P{priority}")
}

/// Formats a priority value with color.
pub fn format_priority_colored(priority: u8) -> String {
    format!("{}", color_theme::priority(format!("P{priority}")))
}

/// Formats a task type for display.
pub fn format_task_type(task_type: TaskType) -> String {
    task_type.to_string()
}

/// Formats a task type with color.
pub fn format_task_type_colored(task_type: TaskType) -> String {
    format!("{}", color_theme::task_type(task_type))
}

/// Formats a task state for display.
pub fn format_state(state: TaskState) -> String {
    state.to_string()
}

/// Formats a task state with color.
pub fn format_state_colored(state: TaskState) -> String {
    match state {
        TaskState::Open => format!("{}", color_theme::status_open("open")),
        TaskState::Blocked => format!("{}", color_theme::status_blocked("blocked")),
        TaskState::Closed => format!("{}", color_theme::status_closed("closed")),
    }
}

/// Formats a count (e.g., "5 tasks", "1 document").
pub fn format_count(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 { format!("1 {singular}") } else { format!("{count} {plural}") }
}

/// Formats a file path for display.
pub fn format_path(path: &str) -> String {
    path.to_string()
}

/// Formats a file path with color.
pub fn format_path_colored(path: &str) -> String {
    format!("{}", color_theme::path(path))
}

/// Outputs a single item or list as JSON.
pub fn output_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Outputs a list as JSON array.
pub fn output_json_array<T: Serialize>(values: &[T]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(values)
}

/// Separator line for pretty output.
pub fn separator_line(width: usize) -> String {
    "-".repeat(width)
}

/// Separator line with color.
pub fn separator_line_colored(width: usize) -> String {
    format!("{}", color_theme::muted("-".repeat(width)))
}

impl OutputFormat {
    /// Creates the appropriate format based on CLI flags.
    pub fn from_flags(json: bool, pretty: bool) -> Self {
        if json {
            OutputFormat::Json
        } else if pretty {
            OutputFormat::Pretty
        } else {
            OutputFormat::Text
        }
    }

    /// Returns true if JSON output is selected.
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Closed => write!(f, "closed"),
            TaskState::Blocked => write!(f, "blocked"),
            TaskState::Open => write!(f, "open"),
        }
    }
}

impl TypeIndicator {
    /// Creates a type indicator for a task.
    pub fn task(priority: u8, closed: bool, task_type: TaskType) -> Self {
        TypeIndicator::Task { priority, closed, task_type }
    }

    /// Creates a type indicator for a knowledge base document.
    pub fn doc() -> Self {
        TypeIndicator::KnowledgeBase
    }
}

impl fmt::Display for TypeIndicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeIndicator::Task { priority, closed: true, .. } => write!(f, "P{priority}/closed"),
            TypeIndicator::Task { priority, closed: false, .. } => write!(f, "P{priority}"),
            TypeIndicator::KnowledgeBase => write!(f, "doc"),
        }
    }
}

impl DocumentRef {
    /// Creates a new document reference for a task.
    pub fn task(
        id: LatticeId,
        name: String,
        description: String,
        priority: u8,
        closed: bool,
        task_type: TaskType,
    ) -> Self {
        Self {
            id,
            name,
            description,
            type_indicator: TypeIndicator::task(priority, closed, task_type),
        }
    }

    /// Creates a new document reference for a knowledge base document.
    pub fn knowledge_base(id: LatticeId, name: String, description: String) -> Self {
        Self { id, name, description, type_indicator: TypeIndicator::doc() }
    }

    /// Formats as text output (no colors).
    pub fn format_text(&self) -> String {
        format!("{}: {} - {} [{}]", self.id, self.name, self.description, self.type_indicator)
    }

    /// Formats as pretty output with colors.
    pub fn format_pretty(&self) -> String {
        let id_str = color_theme::lattice_id(&self.id);
        let name_str = color_theme::bold(&self.name);
        let desc_str = &self.description;
        let type_str = self.format_type_indicator_colored();
        format!("{id_str}: {name_str} - {desc_str} [{type_str}]")
    }

    /// Formats as compact output (ID and name only).
    pub fn format_compact(&self) -> String {
        format!("{}  {}", self.id, self.name)
    }

    /// Formats as compact output with colors.
    pub fn format_compact_pretty(&self) -> String {
        let id_str = color_theme::lattice_id(&self.id);
        let name_str = color_theme::bold(&self.name);
        format!("{id_str}  {name_str}")
    }

    fn format_type_indicator_colored(&self) -> String {
        match &self.type_indicator {
            TypeIndicator::Task { priority, closed: true, .. } => {
                format!(
                    "{}/{}",
                    color_theme::priority(format!("P{priority}")),
                    color_theme::status_closed("closed")
                )
            }
            TypeIndicator::Task { priority, closed: false, .. } => {
                format!("{}", color_theme::priority(format!("P{priority}")))
            }
            TypeIndicator::KnowledgeBase => {
                format!("{}", color_theme::muted("doc"))
            }
        }
    }
}

impl fmt::Display for DocumentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_text())
    }
}

impl From<&DocumentRef> for DocumentRefJson {
    fn from(doc: &DocumentRef) -> Self {
        match &doc.type_indicator {
            TypeIndicator::Task { priority, closed, task_type } => DocumentRefJson {
                id: doc.id.to_string(),
                name: doc.name.clone(),
                description: doc.description.clone(),
                priority: Some(*priority),
                state: Some(if *closed { TaskState::Closed } else { TaskState::Open }),
                task_type: Some(*task_type),
            },
            TypeIndicator::KnowledgeBase => DocumentRefJson {
                id: doc.id.to_string(),
                name: doc.name.clone(),
                description: doc.description.clone(),
                priority: None,
                state: None,
                task_type: None,
            },
        }
    }
}

impl<T> TreeNode<T> {
    /// Creates a new tree node with no children.
    pub fn leaf(data: T) -> Self {
        Self { data, children: Vec::new() }
    }

    /// Creates a new tree node with children.
    pub fn with_children(data: T, children: Vec<TreeNode<T>>) -> Self {
        Self { data, children }
    }

    /// Adds a child node.
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }
}

impl TreeFormatter {
    /// Creates a new tree formatter.
    pub fn new() -> Self {
        Self
    }

    /// Formats a tree to a string using the provided node formatter.
    pub fn format<T, F>(&self, root: &TreeNode<T>, format_node: F) -> String
    where
        F: Fn(&T) -> String + Copy,
    {
        let mut output = String::new();
        self.format_node(&mut output, root, "", true, format_node);
        output
    }

    /// Formats multiple roots as a forest.
    pub fn format_forest<T, F>(&self, roots: &[TreeNode<T>], format_node: F) -> String
    where
        F: Fn(&T) -> String + Copy,
    {
        let mut output = String::new();
        for (i, root) in roots.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            self.format_node(&mut output, root, "", true, format_node);
        }
        output
    }

    fn format_node<T, F>(
        &self,
        output: &mut String,
        node: &TreeNode<T>,
        prefix: &str,
        is_last: bool,
        format_node: F,
    ) where
        F: Fn(&T) -> String + Copy,
    {
        let node_str = format_node(&node.data);
        if prefix.is_empty() {
            let _ = writeln!(output, "{node_str}");
        } else {
            let connector = if is_last { tree_chars::LAST_BRANCH } else { tree_chars::BRANCH };
            let _ = writeln!(
                output,
                "{prefix}{connector}{}{} {node_str}",
                tree_chars::HORIZONTAL,
                tree_chars::HORIZONTAL
            );
        }
        let child_prefix = if prefix.is_empty() {
            String::new()
        } else if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}{}   ", tree_chars::VERTICAL)
        };
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_node(output, child, &child_prefix, is_last_child, format_node);
        }
    }
}

impl Default for TreeFormatter {
    fn default() -> Self {
        Self::new()
    }
}
