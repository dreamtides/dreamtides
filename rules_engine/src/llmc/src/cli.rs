use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "llmc")]
#[command(about = "LLMC v2: Agent Coordination System", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new LLMC workspace
    Init,

    /// Start the LLMC daemon
    Up,

    /// Stop the LLMC daemon
    Down,

    /// Add a new worker to the pool
    Add {
        /// Worker name
        name: String,
    },

    /// Remove a worker and its worktree
    Nuke {
        /// Worker name to remove
        name: String,
    },

    /// Show status of all workers
    Status,

    /// Start a worker on a task
    Start {
        /// Worker name
        worker: String,

        /// Task to assign
        task: String,
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
        /// Worker name
        worker: String,
    },

    /// Reject a worker's work and request changes
    Reject {
        /// Worker name
        worker: String,

        /// Reason for rejection
        reason: String,
    },

    /// Accept a worker's work and merge
    Accept {
        /// Worker name
        worker: String,
    },

    /// Rebase a worker's branch
    Rebase {
        /// Worker name
        worker: String,
    },

    /// Check system health and configuration
    Doctor,
}
