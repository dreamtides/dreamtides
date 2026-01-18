use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use rusqlite::Connection;
use tracing::{debug, error, info, warn};

use crate::claim::stale_cleanup;
use crate::cli::argument_parser::{Command, Lat};
use crate::cli::commands::list_command::list_executor;
use crate::cli::commands::ready_command::ready_executor;
use crate::cli::commands::show_command::show_executor;
use crate::cli::commands::{
    blocked_command, changes_command, claim_command, close_command, create_command, generate_ids,
    search_command, stale_command, stats_command, track_command, tree_command, update_command,
};
use crate::cli::global_options::GlobalOptions;
use crate::config::config_loader;
use crate::config::config_schema::Config;
use crate::error::error_types::LatticeError;
use crate::error::exit_codes;
use crate::git::git_ops::GitOps;
use crate::git::real_git::RealGit;
use crate::index::connection_pool;
use crate::index::reconciliation::reconciliation_coordinator;
use crate::log::log_init::{self, LogConfig, Verbosity};

/// Maximum startup time before emitting a debug warning (100ms).
const STARTUP_PERFORMANCE_BUDGET_MS: u128 = 100;
/// Maximum log file size before rotation (10 MB).
const MAX_LOG_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Result type for Lattice command handlers.
pub type LatticeResult<T> = Result<T, LatticeError>;

/// Shared context passed to all command handlers.
///
/// Contains all state needed by command handlers: Git operations, database
/// connection, configuration, and global options. Created once during startup
/// and threaded through to handlers.
pub struct CommandContext {
    pub git: Box<dyn GitOps>,
    pub conn: Connection,
    pub config: Config,
    pub repo_root: PathBuf,
    pub global: GlobalOptions,
}

/// Executes the parsed CLI arguments and returns the appropriate exit code.
///
/// This is the main entry point from the binary. It:
/// 1. Initializes logging
/// 2. Detects the repository root
/// 3. Runs startup operations (unless `--no-startup`)
/// 4. Dispatches to the appropriate command handler
/// 5. Returns the exit code based on the result
pub fn run(args: Lat) -> ExitCode {
    let verbosity = args.global.verbosity();

    let repo_root = match find_repo_root() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Error: {e}");
            return exit_codes::user_input_error();
        }
    };

    init_logging(&repo_root, verbosity);
    info!(command = ?args.command, "lat command starting");

    let context = match create_context(&repo_root, &args.global) {
        Ok(ctx) => ctx,
        Err(e) => {
            error!(error = %e, "Failed to create command context");
            eprintln!("Error: {e}");
            return ExitCode::from(e.exit_code());
        }
    };

    if !args.global.no_startup {
        if let Err(e) = run_startup_operations(&context) {
            error!(error = %e, "Startup operations failed");
            eprintln!("Error during startup: {e}");
            return ExitCode::from(e.exit_code());
        }
    } else {
        debug!("Skipping startup operations (--no-startup flag)");
    }

    match dispatch_command(context, args.command) {
        Ok(()) => {
            info!("Command completed successfully");
            exit_codes::success()
        }
        Err(e) => {
            error!(error = %e, error_code = e.error_code(), "Command failed");
            if args.global.json {
                print_json_error(&e);
            } else {
                eprintln!("Error: {e}");
            }
            ExitCode::from(e.exit_code())
        }
    }
}

/// Finds the repository root by walking up from the current directory.
pub fn find_repo_root() -> LatticeResult<PathBuf> {
    let cwd = std::env::current_dir().map_err(|e| LatticeError::ReadError {
        path: PathBuf::from("."),
        reason: format!("Failed to get current directory: {e}"),
    })?;

    find_repo_root_from(&cwd)
}

/// Finds the repository root starting from the given path.
pub fn find_repo_root_from(start: &Path) -> LatticeResult<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            debug!(repo_root = %current.display(), "Found repository root");
            return Ok(current);
        }

        if !current.pop() {
            return Err(LatticeError::GitError {
                operation: "find repository".to_string(),
                reason: format!("Not a git repository (or any parent): {}", start.display()),
            });
        }
    }
}

/// Creates the command context with Git operations, database connection, and
/// config.
pub fn create_context(repo_root: &Path, global: &GlobalOptions) -> LatticeResult<CommandContext> {
    let git = Box::new(RealGit::new(repo_root.to_path_buf()));
    let config = config_loader::load_config(Some(repo_root))?;
    config_loader::validate_config(&config)?;

    connection_pool::ensure_lattice_dir(repo_root)?;
    let conn = connection_pool::open_connection(repo_root)?;

    Ok(CommandContext {
        git,
        conn,
        config,
        repo_root: repo_root.to_path_buf(),
        global: global.clone(),
    })
}

/// Runs startup operations before command execution.
///
/// Per appendix_startup_operations.md, this includes:
/// - Index reconciliation
/// - Skill symlink synchronization
/// - Claim cleanup
/// - Log rotation
///
/// All operations are idempotent and safe under concurrent execution.
/// Emits a debug warning if total time exceeds 100ms.
fn run_startup_operations(context: &CommandContext) -> LatticeResult<()> {
    let start = Instant::now();
    info!("Running startup operations");

    run_index_reconciliation(context)?;
    run_skill_symlink_sync(context)?;
    run_claim_cleanup(context)?;
    run_log_rotation(&context.repo_root)?;

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    if elapsed_ms > STARTUP_PERFORMANCE_BUDGET_MS {
        warn!(
            elapsed_ms,
            budget_ms = STARTUP_PERFORMANCE_BUDGET_MS,
            "Startup operations exceeded performance budget"
        );
    } else {
        info!(elapsed_ms, "Startup operations completed");
    }

    Ok(())
}

/// Reconciles the SQLite index with the git repository.
///
/// Fast path (~1ms): Skip if HEAD unchanged and no uncommitted .md changes.
/// Incremental path (~50-500ms): Re-parse modified documents.
/// Full rebuild (seconds): Triggered by missing index or corruption.
fn run_index_reconciliation(context: &CommandContext) -> LatticeResult<()> {
    let result = reconciliation_coordinator::reconcile(
        &context.repo_root,
        context.git.as_ref(),
        &context.conn,
    )?;
    debug!(?result, "Index reconciliation completed");
    Ok(())
}

/// Synchronizes skill symlinks in .claude/skills/.
///
/// Scans index for skill-enabled documents and creates/updates/removes
/// symlinks to match current state.
fn run_skill_symlink_sync(_context: &CommandContext) -> LatticeResult<()> {
    debug!("Skill symlink sync: not yet implemented, skipping");
    Ok(())
}

/// Cleans up stale claims.
///
/// A claim is stale if:
/// - The referenced task no longer exists
/// - The task is in a .closed/ directory
/// - The worktree path no longer exists
/// - The claim is older than the configured threshold (default 7 days)
fn run_claim_cleanup(context: &CommandContext) -> LatticeResult<()> {
    let summary = stale_cleanup::cleanup_stale_claims(
        &context.conn,
        &context.repo_root,
        &context.config.claim,
    )?;

    if summary.total() > 0 {
        debug!(
            released = summary.released.len(),
            kept = summary.kept.len(),
            errors = summary.errors.len(),
            "Claim cleanup completed during startup"
        );
    }
    Ok(())
}

/// Rotates the log file if it exceeds the size limit.
///
/// This is a startup-time rotation check separate from the per-write check
/// in JsonlWriter. Ensures log rotation happens even if the previous session
/// crashed before writing.
fn run_log_rotation(repo_root: &Path) -> LatticeResult<()> {
    let log_path = repo_root.join(".lattice").join("logs.jsonl");

    if !log_path.exists() {
        debug!("Log file does not exist, skipping rotation check");
        return Ok(());
    }

    let metadata = fs::metadata(&log_path).map_err(|e| LatticeError::ReadError {
        path: log_path.clone(),
        reason: format!("Failed to stat log file: {e}"),
    })?;

    if metadata.len() < MAX_LOG_SIZE_BYTES {
        debug!(size_bytes = metadata.len(), "Log file within size limit");
        return Ok(());
    }

    let rotated_path = log_path.with_extension("jsonl.1");
    debug!(
        size_bytes = metadata.len(),
        rotated_path = %rotated_path.display(),
        "Rotating log file"
    );

    fs::rename(&log_path, &rotated_path).map_err(|e| LatticeError::WriteError {
        path: log_path.clone(),
        reason: format!("Failed to rotate log file: {e}"),
    })?;

    info!("Log file rotated");
    Ok(())
}

/// Dispatches to the appropriate command handler.
fn dispatch_command(context: CommandContext, command: Command) -> LatticeResult<()> {
    match command {
        Command::Show(args) => {
            info!("Dispatching to show command");
            show_executor::execute(context, args)
        }
        Command::Create(args) => {
            info!("Dispatching to create command");
            create_command::execute(context, args)
        }
        Command::Update(args) => {
            info!("Dispatching to update command");
            update_command::execute(context, args)
        }
        Command::Close(args) => {
            info!("Dispatching to close command");
            close_command::execute(context, args)
        }
        Command::Reopen(_args) => {
            info!("Dispatching to reopen command");
            Err(LatticeError::OperationNotAllowed {
                reason: "reopen command not yet implemented".to_string(),
            })
        }
        Command::Prune(_args) => {
            info!("Dispatching to prune command");
            Err(LatticeError::OperationNotAllowed {
                reason: "prune command not yet implemented".to_string(),
            })
        }
        Command::List(args) => {
            info!("Dispatching to list command");
            list_executor::execute(context, args)
        }
        Command::Ready(args) => {
            info!("Dispatching to ready command");
            ready_executor::execute(context, args)
        }
        Command::Search(args) => {
            info!("Dispatching to search command");
            search_command::execute(context, args)
        }
        Command::Stale(args) => {
            info!("Dispatching to stale command");
            stale_command::execute(context, args)
        }
        Command::Blocked(args) => {
            info!("Dispatching to blocked command");
            blocked_command::execute(context, args)
        }
        Command::Changes(args) => {
            info!("Dispatching to changes command");
            changes_command::execute(context, args)
        }
        Command::Stats(args) => {
            info!("Dispatching to stats command");
            stats_command::execute(context, args)
        }
        Command::Tree(args) => {
            info!("Dispatching to tree command");
            tree_command::execute(context, args)
        }
        Command::Roots(_args) => {
            info!("Dispatching to roots command");
            Err(LatticeError::OperationNotAllowed {
                reason: "roots command not yet implemented".to_string(),
            })
        }
        Command::Children(_args) => {
            info!("Dispatching to children command");
            Err(LatticeError::OperationNotAllowed {
                reason: "children command not yet implemented".to_string(),
            })
        }
        Command::Dep(_args) => {
            info!("Dispatching to dep command");
            Err(LatticeError::OperationNotAllowed {
                reason: "dep command not yet implemented".to_string(),
            })
        }
        Command::Label(_args) => {
            info!("Dispatching to label command");
            Err(LatticeError::OperationNotAllowed {
                reason: "label command not yet implemented".to_string(),
            })
        }
        Command::LinksFrom(_args) => {
            info!("Dispatching to links-from command");
            Err(LatticeError::OperationNotAllowed {
                reason: "links-from command not yet implemented".to_string(),
            })
        }
        Command::LinksTo(_args) => {
            info!("Dispatching to links-to command");
            Err(LatticeError::OperationNotAllowed {
                reason: "links-to command not yet implemented".to_string(),
            })
        }
        Command::Path(_args) => {
            info!("Dispatching to path command");
            Err(LatticeError::OperationNotAllowed {
                reason: "path command not yet implemented".to_string(),
            })
        }
        Command::Orphans(_args) => {
            info!("Dispatching to orphans command");
            Err(LatticeError::OperationNotAllowed {
                reason: "orphans command not yet implemented".to_string(),
            })
        }
        Command::Impact(_args) => {
            info!("Dispatching to impact command");
            Err(LatticeError::OperationNotAllowed {
                reason: "impact command not yet implemented".to_string(),
            })
        }
        Command::Claim(args) => {
            info!("Dispatching to claim command");
            claim_command::execute(context, args)
        }
        Command::Overview(_args) => {
            info!("Dispatching to overview command");
            Err(LatticeError::OperationNotAllowed {
                reason: "overview command not yet implemented".to_string(),
            })
        }
        Command::Prime(_args) => {
            info!("Dispatching to prime command");
            Err(LatticeError::OperationNotAllowed {
                reason: "prime command not yet implemented".to_string(),
            })
        }
        Command::Track(args) => {
            info!("Dispatching to track command");
            track_command::execute(context, args)
        }
        Command::GenerateIds(args) => {
            info!("Dispatching to generate-ids command");
            generate_ids::execute(context, args)
        }
        Command::Split(_args) => {
            info!("Dispatching to split command");
            Err(LatticeError::OperationNotAllowed {
                reason: "split command not yet implemented".to_string(),
            })
        }
        Command::Mv(_args) => {
            info!("Dispatching to mv command");
            Err(LatticeError::OperationNotAllowed {
                reason: "mv command not yet implemented".to_string(),
            })
        }
        Command::Edit(_args) => {
            info!("Dispatching to edit command");
            Err(LatticeError::OperationNotAllowed {
                reason: "edit command not yet implemented".to_string(),
            })
        }
        Command::Fmt(_args) => {
            info!("Dispatching to fmt command");
            Err(LatticeError::OperationNotAllowed {
                reason: "fmt command not yet implemented".to_string(),
            })
        }
        Command::Check(_args) => {
            info!("Dispatching to check command");
            Err(LatticeError::OperationNotAllowed {
                reason: "check command not yet implemented".to_string(),
            })
        }
        Command::Doctor(_args) => {
            info!("Dispatching to doctor command");
            Err(LatticeError::OperationNotAllowed {
                reason: "doctor command not yet implemented".to_string(),
            })
        }
        Command::Setup(_args) => {
            info!("Dispatching to setup command");
            Err(LatticeError::OperationNotAllowed {
                reason: "setup command not yet implemented".to_string(),
            })
        }
        Command::Completion(_args) => {
            info!("Dispatching to completion command");
            Err(LatticeError::OperationNotAllowed {
                reason: "completion command not yet implemented".to_string(),
            })
        }
        Command::ChaosMonkey(_args) => {
            info!("Dispatching to chaosmonkey command");
            Err(LatticeError::OperationNotAllowed {
                reason: "chaosmonkey command not yet implemented".to_string(),
            })
        }
    }
}

/// Initializes the logging system based on verbosity and repo root.
fn init_logging(repo_root: &Path, verbosity: Verbosity) {
    let lattice_dir = repo_root.join(".lattice");
    let config = if lattice_dir.exists() {
        LogConfig::with_lattice_dir(&lattice_dir).with_verbosity(verbosity)
    } else {
        LogConfig::default().with_verbosity(verbosity)
    };

    log_init::init_logging(config);
}

/// Prints a JSON-formatted error to stderr.
fn print_json_error(error: &LatticeError) {
    let json = serde_json::json!({
        "error_code": error.error_code(),
        "category": error.category().to_string(),
        "message": error.to_string(),
    });

    if let Ok(output) = serde_json::to_string(&json) {
        eprintln!("{output}");
    } else {
        eprintln!("{{\"error\": \"Failed to serialize error\", \"message\": \"{error}\"}}");
    }
}
