use clap::Args;

use crate::cli::shared_options;
use crate::cli::shared_options::{FilterOptions, ReadySortPolicy};
use crate::document::frontmatter_schema::TaskType;

/// Arguments for `lat show`.
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Document IDs to display.
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// Show brief output (ID and name only).
    #[arg(long)]
    pub short: bool,

    /// Show document references.
    #[arg(long)]
    pub refs: bool,

    /// Show preview without full content.
    #[arg(long)]
    pub peek: bool,

    /// Show raw markdown content.
    #[arg(long)]
    pub raw: bool,
}

/// Arguments for `lat ready`.
#[derive(Args, Debug)]
pub struct ReadyArgs {
    #[command(flatten)]
    pub filter: FilterOptions,

    /// Maximum results to return.
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Visual tree display.
    #[arg(long)]
    pub pretty: bool,

    /// Include backlog items (P4).
    #[arg(long)]
    pub include_backlog: bool,

    /// Include claimed tasks.
    #[arg(long)]
    pub include_claimed: bool,

    /// Sort policy for ready work ordering.
    #[arg(long, value_enum, default_value_t)]
    pub sort: ReadySortPolicy,
}

/// Arguments for `lat overview`.
#[derive(Args, Debug)]
pub struct OverviewArgs {
    /// Optional document ID for contextual overview.
    pub id: Option<String>,

    /// Maximum documents to show.
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Filter by task type.
    #[arg(long, short = 't', value_parser = shared_options::parse_task_type)]
    pub r#type: Option<TaskType>,

    /// Path prefix filter.
    #[arg(long)]
    pub path: Option<String>,

    /// Include closed tasks.
    #[arg(long)]
    pub include_closed: bool,

    /// Reset view counts.
    #[arg(long)]
    pub reset_views: bool,
}

/// Arguments for `lat prime`.
#[derive(Args, Debug)]
pub struct PrimeArgs {
    /// Include full document content.
    #[arg(long)]
    pub full: bool,

    /// Export to file.
    #[arg(long)]
    pub export: Option<String>,
}

/// Arguments for `lat claim`.
#[derive(Args, Debug)]
pub struct ClaimArgs {
    /// Task ID to claim.
    pub id: Option<String>,

    /// List all claims.
    #[arg(long)]
    pub list: bool,

    /// Release claim on task.
    #[arg(long)]
    pub release: Option<String>,

    /// Release all claims.
    #[arg(long)]
    pub release_all: bool,

    /// Release claims for a worktree.
    #[arg(long)]
    pub release_worktree: Option<String>,

    /// Garbage collect stale claims.
    #[arg(long)]
    pub gc: bool,
}

/// Arguments for `lat pop`.
///
/// Combines `lat ready`, `lat claim`, and `lat show` into a single operation
/// optimized for AI agents. Finds the highest-priority ready task, claims it,
/// and outputs full context for starting work.
#[derive(Args, Debug)]
pub struct PopArgs {
    #[command(flatten)]
    pub filter: FilterOptions,

    /// Include backlog items (P4).
    #[arg(long)]
    pub include_backlog: bool,

    /// Sort policy for ready work ordering.
    #[arg(long, value_enum, default_value_t)]
    pub sort: ReadySortPolicy,

    /// Show what would be claimed without actually claiming.
    #[arg(long)]
    pub dry_run: bool,

    /// Include raw markdown body (default: true for JSON, false for text).
    #[arg(long)]
    pub raw: bool,

    /// Skip claiming (useful for testing or inspection).
    #[arg(long)]
    pub no_claim: bool,

    /// Maximum number of active claims allowed before failing.
    #[arg(long)]
    pub max_claims: Option<usize>,
}
