use clap::{Args, ValueEnum};

use crate::document::frontmatter_schema::TaskType;

/// Filter options shared across query commands.
#[derive(Args, Debug, Clone, Default)]
pub struct FilterOptions {
    /// Filter by state (open/blocked/closed).
    #[arg(long)]
    pub state: Option<TaskState>,

    /// Include tasks in .closed/ directories.
    #[arg(long)]
    pub include_closed: bool,

    /// Show only closed tasks.
    #[arg(long)]
    pub closed_only: bool,

    /// Exact priority filter.
    #[arg(long, short = 'p')]
    pub priority: Option<u8>,

    /// Minimum priority.
    #[arg(long)]
    pub priority_min: Option<u8>,

    /// Maximum priority.
    #[arg(long)]
    pub priority_max: Option<u8>,

    /// Filter by task type.
    #[arg(long, short = 't', value_parser = parse_task_type)]
    pub r#type: Option<TaskType>,

    /// Must have ALL labels (comma-separated).
    #[arg(long, short = 'l', value_delimiter = ',')]
    pub label: Vec<String>,

    /// Must have ANY label (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub label_any: Vec<String>,

    /// Substring match on name.
    #[arg(long)]
    pub name_contains: Option<String>,

    /// Path prefix filter.
    #[arg(long)]
    pub path: Option<String>,

    /// Created after date (ISO 8601).
    #[arg(long)]
    pub created_after: Option<String>,

    /// Created before date (ISO 8601).
    #[arg(long)]
    pub created_before: Option<String>,

    /// Updated after date (ISO 8601).
    #[arg(long)]
    pub updated_after: Option<String>,

    /// Updated before date (ISO 8601).
    #[arg(long)]
    pub updated_before: Option<String>,

    /// Tasks discovered from specified parent.
    #[arg(long)]
    pub discovered_from: Option<String>,

    /// List only root documents.
    #[arg(long)]
    pub roots_only: bool,

    /// Parent document for scope filtering.
    #[arg(long)]
    pub parent: Option<String>,
}

/// Output limit and sort options.
#[derive(Args, Debug, Clone, Default)]
pub struct OutputOptions {
    /// Maximum results to return.
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Sort by field.
    #[arg(long, value_enum)]
    pub sort: Option<SortField>,

    /// Reverse sort order.
    #[arg(long)]
    pub reverse: bool,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<ListFormat>,

    /// Visual tree display.
    #[arg(long)]
    pub pretty: bool,
}

/// Sort fields for list commands.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Priority,
    Created,
    Updated,
    Name,
}

/// List output formats.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormat {
    Rich,
    Compact,
    Oneline,
}

/// Task state filter values.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Open,
    Blocked,
    Closed,
}

/// Ready command sort policies.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadySortPolicy {
    #[default]
    Hybrid,
    Priority,
    Oldest,
}

/// Shell types for completion generation.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

/// Parses a string into a TaskType.
pub fn parse_task_type(s: &str) -> Result<TaskType, String> {
    s.parse()
}
