use std::io::{self, Write};

use clap::CommandFactory;
use clap_complete::{Shell as ClapShell, generate};
use rusqlite::Connection;
use tracing::info;

use crate::cli::argument_parser::Lat;
use crate::cli::command_dispatch::LatticeResult;
use crate::cli::maintenance_args::CompletionArgs;
use crate::cli::shared_options::Shell;
use crate::error::error_types::LatticeError;
use crate::index::document_queries;

/// Maximum number of IDs to return for completion.
///
/// Limits output to prevent overwhelming shell completion with too many
/// options.
const MAX_COMPLETION_IDS: usize = 100;
/// Bash dynamic completion additions.
///
/// Defines `_lat_complete_ids` function and a wrapper that integrates with the
/// generated completion script.
const BASH_DYNAMIC_COMPLETION: &str = r#"

# Dynamic Lattice ID completion
_lat_complete_ids() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local ids
    ids=$(lat completion --ids --prefix "$cur" 2>/dev/null)
    if [[ -n "$ids" ]]; then
        COMPREPLY=($(compgen -W "$ids" -- "$cur"))
    fi
}

# Override completion for commands that accept Lattice IDs
_lat_with_ids() {
    local cur prev cmd
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Commands that accept Lattice IDs as positional arguments
    case "$prev" in
        show|close|reopen|update|claim|impact|edit|split|mv|track|prune)
            _lat_complete_ids
            return
            ;;
        links-from|links-to)
            _lat_complete_ids
            return
            ;;
    esac

    # For 'dep add' and 'dep remove' subcommands
    if [[ "${COMP_WORDS[1]}" == "dep" ]]; then
        case "${COMP_WORDS[2]}" in
            add|remove|tree)
                _lat_complete_ids
                return
                ;;
        esac
    fi

    # For 'path' command (takes two IDs)
    if [[ "${COMP_WORDS[1]}" == "path" ]]; then
        _lat_complete_ids
        return
    fi

    # Fall back to default completion
    _lat
}

# Register the enhanced completion
complete -F _lat_with_ids lat
"#;
/// Zsh dynamic completion additions.
const ZSH_DYNAMIC_COMPLETION: &str = r#"

# Dynamic Lattice ID completion function
_lat_complete_ids() {
    local -a ids
    ids=(${(f)"$(lat completion --ids --prefix "${words[CURRENT]}" 2>/dev/null)"})
    _describe -t ids 'Lattice ID' ids
}

# Enhanced completion with dynamic IDs
_lat_dynamic() {
    local curcontext="$curcontext" state
    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _lat
            ;;
        args)
            case $words[1] in
                show|close|reopen|update|claim|impact|edit|split|mv|track|prune|links-from|links-to|path)
                    _lat_complete_ids
                    ;;
                dep)
                    case $words[2] in
                        add|remove|tree)
                            _lat_complete_ids
                            ;;
                        *)
                            _lat
                            ;;
                    esac
                    ;;
                *)
                    _lat
                    ;;
            esac
            ;;
    esac
}

compdef _lat_dynamic lat
"#;
/// Fish dynamic completion additions.
const FISH_DYNAMIC_COMPLETION: &str = r#"

# Dynamic Lattice ID completion for commands that accept IDs
function __fish_lat_complete_ids
    lat completion --ids --prefix (commandline -ct) 2>/dev/null
end

# Commands that accept Lattice IDs as their first positional argument
complete -c lat -n "__fish_seen_subcommand_from show close reopen update claim impact edit split mv track prune links-from links-to path" -a "(__fish_lat_complete_ids)" -f

# Dep subcommands that accept IDs
complete -c lat -n "__fish_seen_subcommand_from dep; and __fish_seen_subcommand_from add remove tree" -a "(__fish_lat_complete_ids)" -f
"#;

/// Executes the `lat completion` command.
///
/// Operates in one of two modes based on arguments:
///
/// - **Shell mode** (default): Generates a shell completion script for the
///   specified shell and outputs it to stdout.
/// - **IDs mode** (`--ids`): Outputs Lattice IDs matching the optional prefix,
///   one per line, for dynamic shell completion.
///
/// # Shell mode installation paths
///
/// - **Bash:** `~/.local/share/bash-completion/completions/lat`
/// - **Zsh:** `~/.zfunc/_lat` (ensure `fpath+=~/.zfunc` in `.zshrc`)
/// - **Fish:** `~/.config/fish/completions/lat.fish`
/// - **PowerShell:** See PowerShell documentation for completion installation
/// - **Elvish:** `~/.config/elvish/lib/completions/lat.elv`
pub fn execute(args: CompletionArgs, conn: Option<&Connection>) -> LatticeResult<()> {
    if args.ids {
        execute_ids_completion(args.prefix.as_deref(), conn)
    } else {
        execute_shell_completion(args.shell)
    }
}

/// Generates shell completion script for the given shell to a writer.
///
/// This function generates the base completion script using clap_complete
/// and appends custom dynamic ID completion support for shells that support it.
pub fn generate_to_writer<W: Write>(shell: Shell, out: &mut W) {
    let clap_shell = to_clap_shell(shell);
    let mut cmd = Lat::command();
    generate(clap_shell, &mut cmd, "lat", out);

    match shell {
        Shell::Bash => append_bash_dynamic_completion(out),
        Shell::Zsh => append_zsh_dynamic_completion(out),
        Shell::Fish => append_fish_dynamic_completion(out),
        Shell::PowerShell | Shell::Elvish => {}
    }
}

/// Generates shell completion script for static command completion.
fn execute_shell_completion(shell: Option<Shell>) -> LatticeResult<()> {
    let shell = shell.ok_or_else(|| LatticeError::InvalidArgument {
        message: "Shell type is required when not using --ids".to_string(),
    })?;

    info!(shell = ?shell, "Generating shell completions");
    generate_to_writer(shell, &mut io::stdout());
    info!("Shell completions generated successfully");
    Ok(())
}

/// Outputs Lattice IDs for dynamic shell completion.
fn execute_ids_completion(prefix: Option<&str>, conn: Option<&Connection>) -> LatticeResult<()> {
    let conn = conn.ok_or_else(|| LatticeError::InvalidArgument {
        message: "Database connection required for ID completion".to_string(),
    })?;

    info!(prefix = prefix, "Generating ID completions");
    let ids = document_queries::ids_by_prefix(conn, prefix, MAX_COMPLETION_IDS)?;

    for id in &ids {
        println!("{id}");
    }

    info!(count = ids.len(), "ID completions generated");
    Ok(())
}

/// Appends Bash dynamic ID completion functions.
fn append_bash_dynamic_completion<W: Write>(out: &mut W) {
    let _ = out.write_all(BASH_DYNAMIC_COMPLETION.as_bytes());
}

/// Appends Zsh dynamic ID completion functions.
fn append_zsh_dynamic_completion<W: Write>(out: &mut W) {
    let _ = out.write_all(ZSH_DYNAMIC_COMPLETION.as_bytes());
}

/// Appends Fish dynamic ID completion.
fn append_fish_dynamic_completion<W: Write>(out: &mut W) {
    let _ = out.write_all(FISH_DYNAMIC_COMPLETION.as_bytes());
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
