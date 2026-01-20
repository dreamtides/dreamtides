//! Shell completion script generation using clap_complete.
//!
//! Generates static shell completion scripts for all `lat` subcommands and
//! their flags. The generated scripts provide tab completion for command names,
//! options, and statically-known arguments.

use std::io::{self, Write};

use clap::CommandFactory;
use clap_complete::{Shell as ClapShell, generate};
use tracing::info;

use crate::cli::argument_parser::Lat;
use crate::cli::command_dispatch::LatticeResult;
use crate::cli::maintenance_args::CompletionArgs;
use crate::cli::shared_options::Shell;

/// Executes the `lat completion` command.
///
/// Generates a shell completion script for the specified shell and outputs it
/// to stdout. Users can redirect this output to the appropriate completion file
/// for their shell.
///
/// # Installation paths
///
/// - **Bash:** `~/.local/share/bash-completion/completions/lat`
/// - **Zsh:** `~/.zfunc/_lat` (ensure `fpath+=~/.zfunc` in `.zshrc`)
/// - **Fish:** `~/.config/fish/completions/lat.fish`
/// - **PowerShell:** See PowerShell documentation for completion installation
/// - **Elvish:** `~/.config/elvish/lib/completions/lat.elv`
pub fn execute(args: CompletionArgs) -> LatticeResult<()> {
    info!(shell = ?args.shell, "Generating shell completions");

    generate_to_writer(args.shell, &mut io::stdout());

    info!("Shell completions generated successfully");
    Ok(())
}

/// Generates shell completion script for the given shell to a writer.
///
/// This function is the core completion generation logic, separated from
/// execute to enable testing with arbitrary output destinations.
pub fn generate_to_writer<W: Write>(shell: Shell, out: &mut W) {
    let clap_shell = to_clap_shell(shell);
    let mut cmd = Lat::command();
    generate(clap_shell, &mut cmd, "lat", out);
}

/// Converts our Shell enum to clap_complete's Shell enum.
fn to_clap_shell(shell: Shell) -> ClapShell {
    match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::PowerShell => ClapShell::PowerShell,
        Shell::Elvish => ClapShell::Elvish,
    }
}
