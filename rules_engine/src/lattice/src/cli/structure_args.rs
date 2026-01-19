use clap::{Args, Subcommand};

/// Arguments for `lat tree`.
#[derive(Args, Debug)]
pub struct TreeArgs {
    /// Path to display (defaults to root).
    pub path: Option<String>,

    /// Maximum depth.
    #[arg(long)]
    pub depth: Option<usize>,

    /// Show document counts.
    #[arg(long)]
    pub counts: bool,

    /// Only show task directories.
    #[arg(long)]
    pub tasks_only: bool,

    /// Only show documentation directories.
    #[arg(long)]
    pub docs_only: bool,
}

/// Arguments for `lat roots`.
#[derive(Args, Debug)]
pub struct RootsArgs {}

/// Arguments for `lat children`.
#[derive(Args, Debug)]
pub struct ChildrenArgs {
    /// Root document ID.
    pub root_id: String,

    /// Include nested directories.
    #[arg(long)]
    pub recursive: bool,

    /// Only show tasks.
    #[arg(long)]
    pub tasks: bool,

    /// Only show KB documents.
    #[arg(long)]
    pub docs: bool,
}

/// Arguments for `lat dep`.
#[derive(Args, Debug)]
pub struct DepArgs {
    #[command(subcommand)]
    pub command: DepCommand,
}

/// Dependency subcommands.
#[derive(Subcommand, Debug)]
pub enum DepCommand {
    /// Add dependency (first depends on second).
    Add { id: String, depends_on: String },
    /// Remove dependency relationship.
    Remove { id: String, depends_on: String },
    /// Display dependency tree.
    Tree {
        /// Task ID to display tree for.
        id: String,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Arguments for `lat label`.
#[derive(Args, Debug)]
pub struct LabelArgs {
    #[command(subcommand)]
    pub command: LabelCommand,
}

/// Label subcommands.
#[derive(Subcommand, Debug)]
pub enum LabelCommand {
    /// Add label to documents.
    Add {
        /// Document IDs.
        #[arg(required = true)]
        ids: Vec<String>,
        /// Label to add.
        label: String,
    },
    /// Remove label from documents.
    Remove {
        /// Document IDs.
        #[arg(required = true)]
        ids: Vec<String>,
        /// Label to remove.
        label: String,
    },
    /// List labels on document.
    List { id: String },
    /// List all labels with counts.
    #[command(name = "list-all")]
    ListAll,
}

/// Arguments for `lat links-from`.
#[derive(Args, Debug)]
pub struct LinksFromArgs {
    /// Document ID.
    pub id: String,
}

/// Arguments for `lat links-to`.
#[derive(Args, Debug)]
pub struct LinksToArgs {
    /// Document ID.
    pub id: String,
}

/// Arguments for `lat path`.
#[derive(Args, Debug)]
pub struct PathArgs {
    /// Source document ID.
    pub id1: String,

    /// Target document ID.
    pub id2: String,
}

/// Arguments for `lat orphans`.
#[derive(Args, Debug)]
pub struct OrphansArgs {
    /// Don't report root documents.
    #[arg(long)]
    pub exclude_roots: bool,

    /// Check only under path.
    #[arg(long)]
    pub path: Option<String>,
}

/// Arguments for `lat impact`.
#[derive(Args, Debug)]
pub struct ImpactArgs {
    /// Document ID.
    pub id: String,
}
