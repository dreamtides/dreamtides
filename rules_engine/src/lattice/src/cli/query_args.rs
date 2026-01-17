use clap::Args;

use crate::cli::shared_options;
use crate::cli::shared_options::{FilterOptions, OutputOptions};
use crate::document::frontmatter_schema::TaskType;

/// Arguments for `lat list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    #[command(flatten)]
    pub filter: FilterOptions,

    #[command(flatten)]
    pub output: OutputOptions,
}

/// Arguments for `lat search`.
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query.
    pub query: String,

    /// Maximum results.
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Restrict to path prefix.
    #[arg(long)]
    pub path: Option<String>,

    /// Filter by task type.
    #[arg(long, short = 't', value_parser = shared_options::parse_task_type)]
    pub r#type: Option<TaskType>,
}

/// Arguments for `lat stale`.
#[derive(Args, Debug)]
pub struct StaleArgs {
    /// Staleness threshold in days (default 30).
    #[arg(long, default_value_t = 30)]
    pub days: u32,

    #[command(flatten)]
    pub filter: FilterOptions,

    #[command(flatten)]
    pub output: OutputOptions,
}

/// Arguments for `lat blocked`.
#[derive(Args, Debug)]
pub struct BlockedArgs {
    /// Path prefix filter.
    #[arg(long)]
    pub path: Option<String>,

    /// Maximum results.
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Display blocking tasks.
    #[arg(long)]
    pub show_blockers: bool,
}

/// Arguments for `lat changes`.
#[derive(Args, Debug)]
pub struct ChangesArgs {
    /// Since date or git commit.
    #[arg(long)]
    pub since: Option<String>,
}

/// Arguments for `lat stats`.
#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Restrict to path prefix.
    #[arg(long)]
    pub path: Option<String>,

    /// Activity period in days (default 7).
    #[arg(long, default_value_t = 7)]
    pub period: u32,
}
