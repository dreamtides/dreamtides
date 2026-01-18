use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "llmc")]
#[command(about = "LLMC v2: Agent Coordination System", long_about = None)]
pub struct Cli {
    /// Enable verbose error output (includes stack traces)
    #[arg(long, short, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new LLMC workspace
    Init {
        /// Path to source repository
        #[arg(long)]
        source: Option<PathBuf>,

        /// Target directory for LLMC workspace (default: ~/llmc)
        #[arg(long)]
        target: Option<PathBuf>,

        /// Remove existing directory if present
        #[arg(long)]
        force: bool,
    },

    /// Start the LLMC daemon
    Up {
        /// Disable patrol system
        #[arg(long)]
        no_patrol: bool,

        /// Force cleanup of existing sessions before starting
        #[arg(long)]
        force: bool,
    },

    /// Stop the LLMC daemon
    Down {
        /// Force shutdown (kill sessions immediately)
        #[arg(long)]
        force: bool,

        /// Also kill console sessions (by default, consoles persist across
        /// restarts)
        #[arg(long)]
        kill_consoles: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Add a new worker to the pool
    Add {
        /// Worker name
        name: String,

        /// Model to use for this worker (overrides default)
        #[arg(long)]
        model: Option<String>,

        /// Role prompt for this worker
        #[arg(long)]
        role_prompt: Option<String>,

        /// Exclude this worker from automatic task assignment pool
        #[arg(long)]
        excluded_from_pool: bool,

        /// Enable self-review phase for this worker
        #[arg(long)]
        self_review: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Remove a worker and its worktree
    Nuke {
        /// Worker name to remove
        name: Option<String>,

        /// Remove all workers
        #[arg(long)]
        all: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Show status of all workers
    Status {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Start a worker on a task
    Start {
        /// Worker name (optional, selects first idle worker if not specified)
        #[arg(long)]
        worker: Option<String>,

        /// Worker name prefix (selects first idle worker whose name starts with
        /// prefix)
        #[arg(long)]
        prefix: Option<String>,

        /// Prompt text to assign
        #[arg(long)]
        prompt: Option<String>,

        /// Path to file containing prompt
        #[arg(long)]
        prompt_file: Option<PathBuf>,

        /// Command to execute, using its output as the prompt
        #[arg(long)]
        prompt_cmd: Option<String>,

        /// Enable self-review phase; worker performs self-review before human
        /// review
        #[arg(long)]
        self_review: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Send a message to a worker
    Message {
        /// Worker name
        worker: String,

        /// Message to send (opens $EDITOR if not provided)
        message: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Attach to a worker's or console's TMUX session
    Attach {
        /// Worker or console name (e.g., "adam", "console1", "llmc-console1")
        target: String,
    },

    /// Create a new interactive console session or attach to an existing one
    Console {
        /// Console name (e.g., "console1"); creates if missing, attaches if
        /// exists
        name: Option<String>,
    },

    /// Review a worker's completed work
    Review {
        /// Worker name (optional, reviews oldest pending worker if not
        /// specified)
        worker: Option<String>,

        /// Interface to use for displaying the diff
        #[arg(long, default_value = "difftastic")]
        interface: String,

        /// Show only the names of changed files (no diff content)
        #[arg(long)]
        name_only: bool,

        /// Force review regardless of worker state
        #[arg(long)]
        force: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Reject a worker's work and request changes
    Reject {
        /// Reason for rejection (opens $EDITOR with diff if not provided)
        message: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Accept a worker's work and merge
    Accept {
        /// Worker name (optional, accepts most recently reviewed worker if not
        /// specified)
        worker: Option<String>,

        /// Force accept regardless of worker state
        #[arg(long)]
        force: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Rebase a worker's branch
    Rebase {
        /// Worker name
        worker: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Reset a worker to clean idle state
    Reset {
        /// Worker name to reset
        worker: String,

        /// Skip confirmation
        #[arg(long)]
        yes: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Check system health and configuration
    Doctor {
        /// Attempt to auto-repair detected issues
        #[arg(long)]
        repair: bool,

        /// Skip confirmation prompts (use with --repair)
        #[arg(long)]
        yes: bool,

        /// Rebuild state from filesystem
        #[arg(long)]
        rebuild: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Show the last N lines from a worker's session
    Peek {
        /// Worker name (optional, defaults to most recently active worker)
        worker: Option<String>,

        /// Number of lines to display (default: 10)
        #[arg(short, long, default_value = "10")]
        lines: u32,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Low-level command to grab all changes from a worker and rebase onto
    /// master
    Pick {
        /// Worker name
        worker: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show JSON schema documentation for all commands
    Schema,

    /// Hook callback from Claude Code (internal use)
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Notify daemon that a worker session stopped
    Stop {
        /// Worker name
        #[arg(long)]
        worker: String,
    },
    /// Notify daemon of session start
    SessionStart {
        /// Worker name
        #[arg(long)]
        worker: String,
    },
    /// Notify daemon of session end
    SessionEnd {
        /// Worker name
        #[arg(long)]
        worker: String,
        /// Reason for session end
        #[arg(long, default_value = "unknown")]
        reason: String,
    },
    /// Notify daemon of bash command completion
    PostBash {
        /// Worker name
        #[arg(long)]
        worker: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "defaults.model", "workers.adam.model")
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "defaults.model", "workers.adam.model")
        key: String,
        /// New value
        value: String,
    },
}
