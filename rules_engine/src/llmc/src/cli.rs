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
    },

    /// Remove a worker and its worktree
    Nuke {
        /// Worker name to remove
        name: Option<String>,
        /// Remove all workers
        #[arg(long)]
        all: bool,
        /// Reset worker to idle state instead of removing (kills session,
        /// removes worktree, creates new session and worktree, preserves
        /// config)
        #[arg(long)]
        reset: bool,
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

        /// Prompt text to assign
        #[arg(long)]
        prompt: Option<String>,

        /// Path to file containing prompt
        #[arg(long)]
        prompt_file: Option<PathBuf>,
    },

    /// Send a message to a worker
    Message {
        /// Worker name
        worker: String,

        /// Message to send
        message: String,
    },

    /// Attach to a worker's TMUX session
    Attach {
        /// Worker name
        worker: String,
    },

    /// Review a worker's completed work
    Review {
        /// Worker name (optional, reviews oldest pending worker if not
        /// specified)
        worker: Option<String>,

        /// Interface to use for displaying the diff
        #[arg(long, default_value = "difftastic")]
        interface: String,
    },

    /// Reject a worker's work and request changes
    Reject {
        /// Reason for rejection
        message: String,
    },

    /// Accept a worker's work and merge
    Accept {
        /// Worker name (optional, accepts most recently reviewed worker if not
        /// specified)
        worker: Option<String>,
    },

    /// Rebase a worker's branch
    Rebase {
        /// Worker name
        worker: String,
    },

    /// Reset a worker to clean idle state
    Reset {
        /// Worker name to reset
        worker: String,

        /// Skip confirmation
        #[arg(long)]
        yes: bool,
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
    },

    /// Show the last N lines from a worker's session
    Peek {
        /// Worker name (optional, defaults to most recently active worker)
        worker: Option<String>,

        /// Number of lines to display (default: 10)
        #[arg(short, long, default_value = "10")]
        lines: u32,
    },

    /// Low-level command to grab all changes from a worker and rebase onto
    /// master
    Pick {
        /// Worker name
        worker: String,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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
