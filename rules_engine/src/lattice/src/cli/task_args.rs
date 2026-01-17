use clap::Args;

use crate::cli::shared_options;
use crate::document::frontmatter_schema::TaskType;

/// Arguments for `lat create`.
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Parent directory or explicit path.
    pub parent: String,

    /// Task/document description.
    pub description: String,

    /// Task type (makes this a task instead of KB document).
    #[arg(long, short = 't', value_parser = shared_options::parse_task_type)]
    pub r#type: Option<TaskType>,

    /// Priority (0-4, default 2).
    #[arg(long, short = 'p')]
    pub priority: Option<u8>,

    /// File containing document body.
    #[arg(long)]
    pub body_file: Option<String>,

    /// Labels (comma-separated).
    #[arg(long, short = 'l', value_delimiter = ',')]
    pub labels: Vec<String>,

    /// Dependencies specification.
    #[arg(long)]
    pub deps: Option<String>,
}

/// Arguments for `lat update`.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Task IDs to update.
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// New priority.
    #[arg(long)]
    pub priority: Option<u8>,

    /// New task type.
    #[arg(long, value_parser = shared_options::parse_task_type)]
    pub r#type: Option<TaskType>,

    /// Add labels (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub add_labels: Vec<String>,

    /// Remove labels (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub remove_labels: Vec<String>,
}

/// Arguments for `lat close`.
#[derive(Args, Debug)]
pub struct CloseArgs {
    /// Task IDs to close.
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// Closure reason (appended to body).
    #[arg(long)]
    pub reason: Option<String>,

    /// Preview without moving.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lat reopen`.
#[derive(Args, Debug)]
pub struct ReopenArgs {
    /// Task IDs to reopen.
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// Preview without moving.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lat prune`.
#[derive(Args, Debug)]
pub struct PruneArgs {
    /// Path to prune.
    pub path: Option<String>,

    /// Prune all closed tasks.
    #[arg(long)]
    pub all: bool,

    /// Convert inline links to plain text.
    #[arg(long)]
    pub force: bool,

    /// Preview without deleting.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lat track`.
#[derive(Args, Debug)]
pub struct TrackArgs {
    /// Path to markdown file.
    pub path: String,

    /// Set document name.
    #[arg(long)]
    pub name: Option<String>,

    /// Set description.
    #[arg(long)]
    pub description: Option<String>,

    /// Regenerate ID even if present.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `lat generate-ids`.
#[derive(Args, Debug)]
pub struct GenerateIdsArgs {
    /// Number of IDs to generate (default 10).
    #[arg(long, short = 'n', default_value_t = 10)]
    pub count: usize,
}

/// Arguments for `lat split`.
#[derive(Args, Debug)]
pub struct SplitArgs {
    /// Path to document to split.
    pub path: String,

    /// Output directory for split files.
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Preview without writing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lat mv`.
#[derive(Args, Debug)]
pub struct MvArgs {
    /// Document ID to move.
    pub id: String,

    /// New path.
    pub new_path: String,

    /// Preview without writing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lat edit`.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// Document ID to edit.
    pub id: String,

    /// Edit name only.
    #[arg(long)]
    pub name: bool,

    /// Edit description only.
    #[arg(long)]
    pub description: bool,

    /// Edit body only.
    #[arg(long)]
    pub body: bool,
}
