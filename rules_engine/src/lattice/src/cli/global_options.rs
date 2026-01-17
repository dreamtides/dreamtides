use clap::Args;

use crate::log::log_init::Verbosity;

/// Global command-line options available to all `lat` subcommands.
#[derive(Args, Debug, Clone, Default)]
pub struct GlobalOptions {
    /// Output in JSON format.
    #[arg(long, global = true)]
    pub json: bool,

    /// Show detailed operation log to stderr.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Suppress non-error output.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Skip startup operations (debugging only).
    #[arg(long, global = true, hide = true)]
    pub no_startup: bool,
}

impl GlobalOptions {
    /// Returns the verbosity level based on flags.
    ///
    /// Priority: quiet > verbose (quiet wins if both specified).
    pub fn verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else if self.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }
}
