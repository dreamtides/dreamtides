//! Chaos monkey fuzz testing for Lattice.
//!
//! Executes random sequences of operations to discover bugs by checking
//! invariants after each operation. See appendix_chaos_monkey.md for details.
use std::collections::HashMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tempfile::TempDir;
use tracing::{debug, error, info, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::chaos_invariants::{self, InvariantKind, InvariantViolation};
use crate::cli::global_options::GlobalOptions;
use crate::cli::maintenance_args::ChaosMonkeyArgs;
use crate::config::config_schema::Config;
use crate::error::error_types::LatticeError;
use crate::git::client_config::RealClientIdStore;
use crate::git::real_git::RealGit;
use crate::index::reconciliation::reconciliation_coordinator;
use crate::index::{connection_pool, schema_definition};
/// All available operation types for chaos monkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Create a new document via `lat create`.
    Create,
    /// Update an existing document via `lat update`.
    Update,
    /// Close a task via `lat close`.
    Close,
    /// Reopen a closed task via `lat reopen`.
    Reopen,
    /// Prune closed tasks via `lat prune`.
    Prune,
    /// Move a document via `lat mv`.
    Move,
    /// Search documents via `lat search`.
    Search,
    /// Rebuild index via `lat check --rebuild-index`.
    RebuildIndex,
    /// Create a file directly (bypass lat).
    FilesystemCreate,
    /// Delete a file directly (bypass lat).
    FilesystemDelete,
    /// Modify file contents directly (bypass lat).
    FilesystemModify,
    /// Git add and commit.
    GitCommit,
    /// Git checkout to a branch.
    GitCheckout,
    /// Add a dependency between tasks via `lat dep add`.
    DepAdd,
    /// Remove a dependency between tasks via `lat dep remove`.
    DepRemove,
}
/// Record of an operation executed during chaos monkey run.
#[derive(Debug, Clone)]
pub struct OperationRecord {
    /// Operation number (1-indexed).
    pub number: usize,
    /// Type of operation.
    pub op_type: OperationType,
    /// Human-readable description.
    pub description: String,
    /// Whether the operation succeeded (returned Ok).
    pub succeeded: bool,
    /// Error message if operation returned Err.
    pub error_message: Option<String>,
}
/// Result of a chaos monkey run.
#[derive(Debug)]
pub struct ChaosMonkeyResult {
    /// Seed used for this run.
    pub seed: u64,
    /// Total operations executed.
    pub operations_completed: usize,
    /// Maximum operations configured.
    pub max_ops: usize,
    /// Whether the run completed successfully (no invariant violations).
    pub success: bool,
    /// Invariant violation if one occurred.
    pub violation: Option<InvariantViolation>,
    /// The operation that triggered the violation (if any).
    pub failing_operation: Option<OperationRecord>,
    /// History of all operations executed.
    pub operation_history: Vec<OperationRecord>,
    /// Path to preserved repository for debugging (only set on failure).
    pub preserved_repo_path: Option<PathBuf>,
    /// Git log from repository at time of failure.
    pub git_log: Option<String>,
}
/// State for a chaos monkey run.
pub struct ChaosMonkeyState {
    /// Random number generator.
    rng: StdRng,
    /// Seed used for the RNG.
    seed: u64,
    /// Temp directory containing the test repository (None if preserved).
    temp_dir: Option<TempDir>,
    /// Path to the test repository.
    repo_root: PathBuf,
    /// Operation types to run.
    enabled_operations: Vec<OperationType>,
    /// Maximum operations to run.
    max_ops: usize,
    /// Stop before the last (failing) operation.
    stop_before_last: bool,
    /// History of operations.
    operation_history: Vec<OperationRecord>,
    /// Current operation number.
    current_op: usize,
}
/// Executes the `lat chaosmonkey` command.
pub fn execute(_context: CommandContext, args: ChaosMonkeyArgs) -> LatticeResult<()> {
    info!(
        seed = ? args.seed, max_ops = args.max_ops, operations = ? args.operations,
        exclude = ? args.exclude, stop_before_last = args.stop_before_last,
        "Starting chaos monkey"
    );
    let mut state = ChaosMonkeyState::new(&args)?;
    println!("🐒 Chaos Monkey starting");
    println!("   Seed: {}", state.seed());
    println!("   Max ops: {}", state.max_ops());
    println!("   Repository: {}", state.repo_root().display());
    println!();
    let result = run_chaos_loop(&mut state)?;
    print_result(&result);
    if result.success {
        Ok(())
    } else {
        Err(LatticeError::OperationNotAllowed {
            reason: format!(
                "Invariant violation after {} operations: {}",
                result.operations_completed,
                result
                    .violation
                    .as_ref()
                    .map(|v| v.description.clone())
                    .unwrap_or_else(|| "unknown".to_string())
            ),
        })
    }
}
impl OperationType {
    /// Returns all available operation types.
    pub fn all() -> Vec<OperationType> {
        vec![
            OperationType::Create,
            OperationType::Update,
            OperationType::Close,
            OperationType::Reopen,
            OperationType::Prune,
            OperationType::Move,
            OperationType::Search,
            OperationType::RebuildIndex,
            OperationType::FilesystemCreate,
            OperationType::FilesystemDelete,
            OperationType::FilesystemModify,
            OperationType::GitCommit,
            OperationType::GitCheckout,
            OperationType::DepAdd,
            OperationType::DepRemove,
        ]
    }

    /// Returns the string name for this operation type.
    pub fn name(&self) -> &'static str {
        match self {
            OperationType::Create => "create",
            OperationType::Update => "update",
            OperationType::Close => "close",
            OperationType::Reopen => "reopen",
            OperationType::Prune => "prune",
            OperationType::Move => "move",
            OperationType::Search => "search",
            OperationType::RebuildIndex => "rebuild-index",
            OperationType::FilesystemCreate => "fs-create",
            OperationType::FilesystemDelete => "fs-delete",
            OperationType::FilesystemModify => "fs-modify",
            OperationType::GitCommit => "git-commit",
            OperationType::GitCheckout => "git-checkout",
            OperationType::DepAdd => "dep-add",
            OperationType::DepRemove => "dep-remove",
        }
    }

    /// Parses an operation type from a string.
    pub fn from_name(name: &str) -> Option<OperationType> {
        match name.to_lowercase().as_str() {
            "create" => Some(OperationType::Create),
            "update" => Some(OperationType::Update),
            "close" => Some(OperationType::Close),
            "reopen" => Some(OperationType::Reopen),
            "prune" => Some(OperationType::Prune),
            "move" | "mv" => Some(OperationType::Move),
            "search" => Some(OperationType::Search),
            "rebuild-index" | "rebuildindex" => Some(OperationType::RebuildIndex),
            "fs-create" | "fscreate" | "filesystem-create" => Some(OperationType::FilesystemCreate),
            "fs-delete" | "fsdelete" | "filesystem-delete" => Some(OperationType::FilesystemDelete),
            "fs-modify" | "fsmodify" | "filesystem-modify" => Some(OperationType::FilesystemModify),
            "git-commit" | "gitcommit" => Some(OperationType::GitCommit),
            "git-checkout" | "gitcheckout" => Some(OperationType::GitCheckout),
            "dep-add" | "depadd" => Some(OperationType::DepAdd),
            "dep-remove" | "depremove" => Some(OperationType::DepRemove),
            _ => None,
        }
    }
}
impl ChaosMonkeyState {
    /// Creates a new chaos monkey state with the given configuration.
    pub fn new(args: &ChaosMonkeyArgs) -> LatticeResult<Self> {
        let seed = args.seed.unwrap_or_else(|| {
            let mut rng = rand::rng();
            rng.random()
        });
        let rng = StdRng::seed_from_u64(seed);
        info!(seed, "Initializing chaos monkey");
        let enabled_operations = resolve_operations(&args.operations, &args.exclude)?;
        debug!(?enabled_operations, "Enabled operations");
        let temp_dir = TempDir::new().map_err(|e| LatticeError::WriteError {
            path: PathBuf::from("/tmp"),
            reason: format!("Failed to create temp directory: {e}"),
        })?;
        let repo_root = temp_dir.path().to_path_buf();
        info!(repo_root = % repo_root.display(), "Created test repository directory");
        initialize_test_repository(&repo_root)?;
        let max_ops = args.max_ops;
        Ok(Self {
            rng,
            seed,
            temp_dir: Some(temp_dir),
            repo_root,
            enabled_operations,
            max_ops,
            stop_before_last: args.stop_before_last,
            operation_history: Vec::new(),
            current_op: 0,
        })
    }

    /// Returns the seed for this run.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the repository root path.
    pub fn repo_root(&self) -> &PathBuf {
        &self.repo_root
    }

    /// Returns the number of operations completed.
    pub fn operations_completed(&self) -> usize {
        self.current_op
    }

    /// Selects a random operation type.
    pub fn select_operation(&mut self) -> OperationType {
        let index = self.rng.random_range(0..self.enabled_operations.len());
        self.enabled_operations[index]
    }

    /// Generates a random number in the given range.
    pub fn random_range(&mut self, min: usize, max: usize) -> usize {
        self.rng.random_range(min..max)
    }

    /// Generates a random boolean.
    pub fn random_bool(&mut self) -> bool {
        self.rng.random_bool(0.5)
    }

    /// Creates a CommandContext for the test repository.
    pub fn create_context(&self) -> LatticeResult<CommandContext> {
        let git = Box::new(RealGit::new(self.repo_root.clone()));
        let config = Config::default();
        let conn = connection_pool::open_connection(&self.repo_root)?;
        let global = GlobalOptions::default();
        Ok(CommandContext {
            git,
            conn,
            config,
            repo_root: self.repo_root.clone(),
            global,
            client_id_store: Box::new(RealClientIdStore::new()),
        })
    }

    /// Records an operation result.
    pub fn record_operation(
        &mut self,
        op_type: OperationType,
        description: String,
        result: &Result<(), LatticeError>,
    ) {
        self.current_op += 1;
        let record = OperationRecord {
            number: self.current_op,
            op_type,
            description,
            succeeded: result.is_ok(),
            error_message: result.as_ref().err().map(ToString::to_string),
        };
        debug!(
            op_number = record.number,
            op_type = op_type.name(),
            succeeded = record.succeeded,
            "Operation completed"
        );
        self.operation_history.push(record);
    }

    /// Returns the operation history.
    pub fn operation_history(&self) -> &[OperationRecord] {
        &self.operation_history
    }

    /// Returns whether we should stop before this operation.
    pub fn should_stop_before(&self) -> bool {
        self.stop_before_last && self.current_op > 0
    }

    /// Returns whether we've reached the maximum operations.
    pub fn at_max_ops(&self) -> bool {
        self.current_op >= self.max_ops
    }

    /// Returns the maximum operations.
    pub fn max_ops(&self) -> usize {
        self.max_ops
    }

    /// Returns the last operation if any.
    pub fn last_operation(&self) -> Option<&OperationRecord> {
        self.operation_history.last()
    }

    /// Preserves the repository for debugging by preventing temp directory
    /// cleanup. Returns the path where the repository is preserved.
    pub fn preserve_repo(&mut self) -> Option<PathBuf> {
        self.temp_dir.take().map(TempDir::keep)
    }

    /// Gets recent git log for debugging context.
    pub fn get_git_log(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["log", "--oneline", "-10"])
            .current_dir(&self.repo_root)
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }
}
/// Main chaos monkey loop.
fn run_chaos_loop(state: &mut ChaosMonkeyState) -> LatticeResult<ChaosMonkeyResult> {
    loop {
        if state.at_max_ops() {
            info!(
                operations = state.operations_completed(),
                "Reached maximum operations, success!"
            );
            return Ok(ChaosMonkeyResult {
                seed: state.seed(),
                operations_completed: state.operations_completed(),
                max_ops: state.max_ops(),
                success: true,
                violation: None,
                failing_operation: None,
                operation_history: state.operation_history().to_vec(),
                preserved_repo_path: None,
                git_log: None,
            });
        }
        let op_type = state.select_operation();
        debug!(op_type = op_type.name(), "Selected operation");
        let result = execute_operation_with_panic_capture(state, op_type);
        if let Err(violation) = result {
            let git_log = state.get_git_log();
            let preserved_path = state.preserve_repo();
            if state.should_stop_before() {
                warn!(
                    op_number = state.operations_completed(),
                    "Stopping before failing operation (--stop-before-last)"
                );
                return Ok(ChaosMonkeyResult {
                    seed: state.seed(),
                    operations_completed: state.operations_completed(),
                    max_ops: state.max_ops(),
                    success: false,
                    violation: Some(violation),
                    failing_operation: state.last_operation().cloned(),
                    operation_history: state.operation_history().to_vec(),
                    preserved_repo_path: preserved_path,
                    git_log,
                });
            }
            error!(
                op_number = state.operations_completed(), invariant = violation.invariant
                .name(), description = % violation.description,
                "Invariant violation detected!"
            );
            return Ok(ChaosMonkeyResult {
                seed: state.seed(),
                operations_completed: state.operations_completed(),
                max_ops: state.max_ops(),
                success: false,
                violation: Some(violation),
                failing_operation: state.last_operation().cloned(),
                operation_history: state.operation_history().to_vec(),
                preserved_repo_path: preserved_path,
                git_log,
            });
        }
        if state.operations_completed().is_multiple_of(100) {
            println!("   Progress: {} operations completed", state.operations_completed());
        }
    }
}
/// Executes an operation with panic capture.
fn execute_operation_with_panic_capture(
    state: &mut ChaosMonkeyState,
    op_type: OperationType,
) -> Result<(), InvariantViolation> {
    let panic_result =
        panic::catch_unwind(AssertUnwindSafe(|| execute_single_operation(state, op_type)));
    match panic_result {
        Ok(op_result) => {
            state.record_operation(
                op_type,
                format!("Executed {} operation", op_type.name()),
                &op_result,
            );
            check_invariants(state)
        }
        Err(panic_info) => {
            let message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            error!(op_type = op_type.name(), message, "Operation panicked!");
            Err(InvariantViolation {
                invariant: InvariantKind::Panic,
                description: format!("Panic during {} operation: {}", op_type.name(), message),
                affected_paths: vec![],
                affected_ids: vec![],
            })
        }
    }
}
/// Executes a single operation by dispatching to the appropriate generator.
fn execute_single_operation(
    state: &mut ChaosMonkeyState,
    op_type: OperationType,
) -> Result<(), LatticeError> {
    super::chaos_generators::execute_operation(state, op_type)
}
/// Checks all invariants after an operation.
fn check_invariants(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let conn =
        connection_pool::open_connection(&state.repo_root).map_err(|e| InvariantViolation {
            invariant: InvariantKind::IndexHasOrphanedId,
            description: format!("Failed to open connection for invariant check: {e}"),
            affected_paths: vec![],
            affected_ids: vec![],
        })?;
    chaos_invariants::check_all(&conn, &state.repo_root)
}
/// Resolves the final set of operations to run based on includes and excludes.
fn resolve_operations(
    includes: &[String],
    excludes: &[String],
) -> LatticeResult<Vec<OperationType>> {
    let mut operations = if includes.is_empty() {
        OperationType::all()
    } else {
        let mut ops = Vec::new();
        for name in includes {
            let op = OperationType::from_name(name).ok_or_else(|| {
                LatticeError::InvalidArgument { message: format!("Unknown operation type: {name}") }
            })?;
            ops.push(op);
        }
        ops
    };
    for name in excludes {
        let op = OperationType::from_name(name).ok_or_else(|| LatticeError::InvalidArgument {
            message: format!("Unknown operation type: {name}"),
        })?;
        operations.retain(|o| *o != op);
    }
    if operations.is_empty() {
        return Err(LatticeError::InvalidArgument {
            message: "No operations enabled after filtering".to_string(),
        });
    }
    Ok(operations)
}
/// Initializes a fresh test repository.
fn initialize_test_repository(repo_root: &Path) -> LatticeResult<()> {
    info!(repo_root = % repo_root.display(), "Initializing test repository");
    run_git_command(repo_root, &["init"], "init")?;
    configure_git_user(repo_root)?;
    connection_pool::ensure_lattice_dir(repo_root)?;
    let conn = connection_pool::open_connection(repo_root)?;
    schema_definition::create_schema(&conn)?;
    create_initial_structure(repo_root)?;
    let git = RealGit::new(repo_root.to_path_buf());
    reconciliation_coordinator::reconcile(repo_root, &git, &conn)?;
    commit_initial_state(repo_root)?;
    info!("Test repository initialized successfully");
    Ok(())
}
/// Runs a git command and returns an error if it fails.
fn run_git_command(repo_root: &Path, args: &[&str], operation: &str) -> LatticeResult<()> {
    let output = Command::new("git").args(args).current_dir(repo_root).output().map_err(|e| {
        LatticeError::GitError { operation: operation.to_string(), reason: format!("Failed: {e}") }
    })?;
    if !output.status.success() {
        return Err(LatticeError::GitError {
            operation: operation.to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}
/// Configures git user for commits.
fn configure_git_user(repo_root: &Path) -> LatticeResult<()> {
    run_git_command(repo_root, &["config", "user.email", "chaosmonkey@lattice.test"], "config")?;
    run_git_command(repo_root, &["config", "user.name", "Chaos Monkey"], "config")
}
/// Creates initial directory structure with a root document.
fn create_initial_structure(repo_root: &Path) -> LatticeResult<()> {
    let project_dir = repo_root.join("project");
    fs::create_dir_all(&project_dir).map_err(|e| LatticeError::WriteError {
        path: project_dir.clone(),
        reason: format!("Failed to create project directory: {e}"),
    })?;
    fs::create_dir_all(project_dir.join("tasks")).map_err(|e| LatticeError::WriteError {
        path: project_dir.join("tasks"),
        reason: format!("Failed to create tasks directory: {e}"),
    })?;
    fs::create_dir_all(project_dir.join("docs")).map_err(|e| LatticeError::WriteError {
        path: project_dir.join("docs"),
        reason: format!("Failed to create docs directory: {e}"),
    })?;
    let root_doc = repo_root.join("project/project.md");
    let root_content = r#"---
lattice-id: LAAAWQN
name: project
description: Root project document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

# Project

This is the root project document.
"#;
    fs::write(&root_doc, root_content).map_err(|e| LatticeError::WriteError {
        path: root_doc,
        reason: format!("Failed to write root document: {e}"),
    })?;
    info!("Created initial project structure");
    Ok(())
}
/// Commits the initial state.
fn commit_initial_state(repo_root: &Path) -> LatticeResult<()> {
    run_git_command(repo_root, &["add", "."], "add")?;
    run_git_command(repo_root, &["commit", "-m", "Initial commit"], "commit")?;
    info!("Created initial commit");
    Ok(())
}
/// Prints the chaos monkey result.
fn print_result(result: &ChaosMonkeyResult) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if result.success {
        print_success_result(result);
    } else {
        print_failure_result(result);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Prints a successful chaos monkey run result with statistics.
fn print_success_result(result: &ChaosMonkeyResult) {
    println!("✅ CHAOS MONKEY SUCCESS");
    println!();
    println!("Seed: {}", result.seed);
    println!("Operations: {}/{}", result.operations_completed, result.max_ops);
    println!();
    println!("Operation distribution:");
    print_operation_statistics(&result.operation_history);
}

/// Prints a failed chaos monkey run result with debugging information.
fn print_failure_result(result: &ChaosMonkeyResult) {
    println!("❌ CHAOS MONKEY FAILURE");
    println!();
    println!("Seed: {}", result.seed);
    println!("Operations: {}", result.operations_completed);
    if let Some(ref op) = result.failing_operation {
        println!("Failed at: {} (operation #{})", op.description, op.number);
    }
    println!();
    if let Some(ref violation) = result.violation {
        println!("Invariant violated: {}", violation.invariant.name());
        println!("  {}", violation.description);
        if !violation.affected_ids.is_empty() {
            for id in &violation.affected_ids {
                println!("  ID: {id}");
            }
        }
        if !violation.affected_paths.is_empty() {
            for path in &violation.affected_paths {
                println!("  Path: {}", path.display());
            }
        }
    }
    println!();
    println!(
        "Reproduce: lat chaosmonkey --seed {} --max-ops {}",
        result.seed,
        result.operations_completed + 1
    );
    println!(
        "Debug:     lat chaosmonkey --seed {} --max-ops {} --stop-before-last",
        result.seed, result.operations_completed
    );
    if let Some(ref path) = result.preserved_repo_path {
        println!();
        println!("Repository preserved at: {}", path.display());
    }
    if let Some(ref git_log) = result.git_log {
        println!();
        println!("Git history:");
        for line in git_log.lines().take(10) {
            println!("  {line}");
        }
    }
}

/// Computes and prints operation type distribution statistics.
fn print_operation_statistics(history: &[OperationRecord]) {
    let mut counts: HashMap<OperationType, (usize, usize)> = HashMap::new();
    for record in history {
        let entry = counts.entry(record.op_type).or_insert((0, 0));
        entry.0 += 1;
        if record.succeeded {
            entry.1 += 1;
        }
    }
    let mut sorted_ops: Vec<_> = counts.into_iter().collect();
    sorted_ops.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    for (op_type, (total, succeeded)) in sorted_ops {
        #[expect(clippy::cast_precision_loss, reason = "precise percentage not required")]
        let success_rate =
            if total > 0 { ((succeeded as f64 / total as f64) * 100.0) as usize } else { 0 };
        println!(
            "  {}: {} ({} succeeded, {}% success rate)",
            op_type.name(),
            total,
            succeeded,
            success_rate
        );
    }
}
