use clap::{Parser, Subcommand};

use crate::cli::global_options::GlobalOptions;
use crate::cli::maintenance_args::{
    ChaosMonkeyArgs, CheckArgs, CompletionArgs, DoctorArgs, FmtArgs, SetupArgs,
};
use crate::cli::query_args::{
    BlockedArgs, ChangesArgs, ListArgs, SearchArgs, StaleArgs, StatsArgs,
};
use crate::cli::structure_args::{
    ChildrenArgs, DepArgs, ImpactArgs, LabelArgs, LinksFromArgs, LinksToArgs, OrphansArgs,
    PathArgs, RootsArgs, TreeArgs,
};
use crate::cli::task_args::{
    CloseArgs, CreateArgs, EditArgs, GenerateIdsArgs, MvArgs, PruneArgs, ReopenArgs, SplitArgs,
    TrackArgs, UpdateArgs,
};
use crate::cli::workflow_args::{ClaimArgs, OverviewArgs, PopArgs, PrimeArgs, ReadyArgs, ShowArgs};

/// Lattice document management system.
///
/// A unified knowledge base and task tracking system built on markdown files.
#[derive(Parser, Debug)]
#[command(name = "lat", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Lat {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Command,
}

/// All available `lat` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Display document details.
    Show(ShowArgs),
    /// Create a new document or task.
    Create(CreateArgs),
    /// Modify existing tasks.
    Update(UpdateArgs),
    /// Close tasks by moving to .closed/ directory.
    Close(CloseArgs),
    /// Reopen closed tasks.
    Reopen(ReopenArgs),
    /// Permanently delete closed tasks.
    Prune(PruneArgs),
    /// Search and filter documents.
    List(ListArgs),
    /// Find ready work (tasks not blocked or closed).
    Ready(ReadyArgs),
    /// Keyword search across document content.
    Search(SearchArgs),
    /// Find tasks not updated recently.
    Stale(StaleArgs),
    /// Show tasks with unresolved blockers.
    Blocked(BlockedArgs),
    /// Show documents changed since a point in time.
    Changes(ChangesArgs),
    /// Display project statistics.
    Stats(StatsArgs),
    /// Display directory structure with documents.
    Tree(TreeArgs),
    /// List all root documents.
    Roots(RootsArgs),
    /// List documents under a root's directory.
    Children(ChildrenArgs),
    /// Manage dependencies.
    Dep(DepArgs),
    /// Manage labels.
    Label(LabelArgs),
    /// Show documents this document links to.
    #[command(name = "links-from")]
    LinksFrom(LinksFromArgs),
    /// Show documents that link to this document.
    #[command(name = "links-to")]
    LinksTo(LinksToArgs),
    /// Find shortest path between documents.
    Path(PathArgs),
    /// Find documents with no incoming links.
    Orphans(OrphansArgs),
    /// Analyze what would be affected by changes.
    Impact(ImpactArgs),
    /// Mark task as locally in progress.
    Claim(ClaimArgs),
    /// Find highest-priority ready task, claim it, and show context.
    ///
    /// Combines `lat ready`, `lat claim`, and `lat show` into a single
    /// operation optimized for AI agents starting work on tasks.
    Pop(PopArgs),
    /// Show critical documents for context.
    Overview(OverviewArgs),
    /// Output AI workflow context.
    Prime(PrimeArgs),
    /// Add Lattice tracking to existing markdown.
    Track(TrackArgs),
    /// Pre-allocate IDs for offline authoring.
    #[command(name = "generate-ids")]
    GenerateIds(GenerateIdsArgs),
    /// Split document by top-level sections.
    Split(SplitArgs),
    /// Move document to new location.
    Mv(MvArgs),
    /// Open document in editor.
    Edit(EditArgs),
    /// Format documents and normalize links.
    Fmt(FmtArgs),
    /// Validate documents and repository.
    Check(CheckArgs),
    /// Diagnose system health issues.
    Doctor(DoctorArgs),
    /// Install Claude Code hooks and configuration.
    Setup(SetupArgs),
    /// Generate shell completion scripts.
    Completion(CompletionArgs),
    /// Run fuzz testing.
    #[command(name = "chaosmonkey")]
    ChaosMonkey(ChaosMonkeyArgs),
    /// Handle MCP (Model Context Protocol) tool invocations.
    ///
    /// This command is invoked by Claude Code, not directly by users.
    /// It reads a JSON-RPC request from stdin and writes a response to stdout.
    #[command(hide = true)]
    Mcp,
}

/// Parses command-line arguments using Clap.
pub fn parse() -> Lat {
    Lat::parse()
}

/// Parses arguments from an iterator (useful for testing).
pub fn parse_from<I, T>(args: I) -> Lat
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Lat::parse_from(args)
}
