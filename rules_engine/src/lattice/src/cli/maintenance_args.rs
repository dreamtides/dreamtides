use clap::{Args, Subcommand};

use crate::cli::shared_options::Shell;

/// Arguments for `lat fmt`.
#[derive(Args, Debug)]
pub struct FmtArgs {
    /// Only format files under this path.
    #[arg(long)]
    pub path: Option<String>,

    /// Check without modifying (exit 1 if changes needed).
    #[arg(long)]
    pub check: bool,

    /// Override text wrap column (default 80).
    #[arg(long)]
    pub line_width: Option<usize>,
}

/// Arguments for `lat check`.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Only check files under this path.
    #[arg(long)]
    pub path: Option<String>,

    /// Show only errors, not warnings.
    #[arg(long)]
    pub errors_only: bool,

    /// Automatically fix issues.
    #[arg(long)]
    pub fix: bool,

    /// Check only staged files.
    #[arg(long)]
    pub staged_only: bool,

    /// Force index rebuild.
    #[arg(long)]
    pub rebuild_index: bool,
}

/// Arguments for `lat doctor`.
#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Automatically repair issues.
    #[arg(long)]
    pub fix: bool,

    /// Preview fixes without applying.
    #[arg(long)]
    pub dry_run: bool,

    /// Run additional integrity checks.
    #[arg(long)]
    pub deep: bool,

    /// Only show warnings and errors.
    #[arg(long, short = 'q')]
    pub quiet: bool,
}

/// Arguments for `lat setup`.
#[derive(Args, Debug)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub command: SetupCommand,
}

/// Setup subcommands.
#[derive(Subcommand, Debug)]
pub enum SetupCommand {
    /// Install Claude Code hooks.
    Claude {
        /// Check installation status.
        #[arg(long)]
        check: bool,
        /// Remove hooks.
        #[arg(long)]
        remove: bool,
        /// Project-specific installation.
        #[arg(long)]
        project: bool,
    },
}

/// Arguments for `lat completion`.
#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for.
    pub shell: Shell,
}

/// Arguments for `lat chaosmonkey`.
#[derive(Args, Debug)]
pub struct ChaosMonkeyArgs {
    /// Random seed for reproducibility.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Maximum operations to run.
    #[arg(long)]
    pub max_ops: Option<usize>,

    /// Operations to run (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub operations: Vec<String>,

    /// Stop before the last (failing) operation.
    #[arg(long)]
    pub stop_before_last: bool,
}
