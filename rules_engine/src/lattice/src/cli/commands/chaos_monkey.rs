use std::collections::{HashMap, HashSet};
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tempfile::TempDir;
use tracing::{debug, error, info, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::global_options::GlobalOptions;
use crate::cli::maintenance_args::ChaosMonkeyArgs;
use crate::config::config_schema::Config;
use crate::document::document_reader;
use crate::error::error_types::LatticeError;
use crate::git::client_config::RealClientIdStore;
use crate::git::real_git::RealGit;
use crate::id::lattice_id::LatticeId;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_types::DocumentRow;
use crate::index::reconciliation::reconciliation_coordinator;
use crate::index::{connection_pool, document_queries, schema_definition};

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
}

/// Details of an invariant violation detected by chaos monkey.
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    /// Which invariant was violated.
    pub invariant: InvariantKind,
    /// Human-readable description of the violation.
    pub description: String,
    /// Relevant file paths, if any.
    pub affected_paths: Vec<PathBuf>,
    /// Relevant IDs, if any.
    pub affected_ids: Vec<String>,
}

/// Types of invariants checked by chaos monkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantKind {
    /// Index contains ID not in filesystem.
    IndexHasOrphanedId,
    /// Filesystem has document not in index.
    FilesystemHasUnindexedDocument,
    /// Two files share the same Lattice ID.
    DuplicateId,
    /// ID in index doesn't match Lattice ID format.
    MalformedIdInIndex,
    /// A panic occurred during operation.
    Panic,
    /// Index is_closed doesn't match path.
    ClosedStateInconsistency,
    /// Index is_root doesn't match filename.
    RootStateInconsistency,
    /// Index in_tasks_dir or in_docs_dir doesn't match path.
    DirectoryStateInconsistency,
    /// Git operation failed unexpectedly.
    GitOperationFailed,
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
}

/// State for a chaos monkey run.
pub struct ChaosMonkeyState {
    /// Random number generator.
    rng: StdRng,
    /// Seed used for the RNG.
    seed: u64,
    /// Temp directory containing the test repository.
    _temp_dir: TempDir,
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
///
/// Runs randomized operations on a fresh test repository until an invariant
/// violation is detected or the maximum number of operations is reached.
pub fn execute(_context: CommandContext, args: ChaosMonkeyArgs) -> LatticeResult<()> {
    info!(
        seed = ?args.seed,
        max_ops = args.max_ops,
        operations = ?args.operations,
        exclude = ?args.exclude,
        stop_before_last = args.stop_before_last,
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
            _ => None,
        }
    }
}

impl InvariantKind {
    /// Human-readable name for this invariant.
    pub fn name(&self) -> &'static str {
        match self {
            InvariantKind::IndexHasOrphanedId => "index-orphaned-id",
            InvariantKind::FilesystemHasUnindexedDocument => "filesystem-unindexed",
            InvariantKind::DuplicateId => "duplicate-id",
            InvariantKind::MalformedIdInIndex => "malformed-id",
            InvariantKind::Panic => "panic",
            InvariantKind::ClosedStateInconsistency => "closed-state-mismatch",
            InvariantKind::RootStateInconsistency => "root-state-mismatch",
            InvariantKind::DirectoryStateInconsistency => "directory-state-mismatch",
            InvariantKind::GitOperationFailed => "git-operation-failed",
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

        info!(repo_root = %repo_root.display(), "Created test repository directory");

        initialize_test_repository(&repo_root)?;

        let max_ops = args.max_ops;

        Ok(Self {
            rng,
            seed,
            _temp_dir: temp_dir,
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
            });
        }

        let op_type = state.select_operation();
        debug!(op_type = op_type.name(), "Selected operation");

        let result = execute_operation_with_panic_capture(state, op_type);

        if let Err(violation) = result {
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
                });
            }

            error!(
                op_number = state.operations_completed(),
                invariant = violation.invariant.name(),
                description = %violation.description,
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
            });
        }

        if state.operations_completed().is_multiple_of(100) {
            println!("   Progress: {} operations completed", state.operations_completed());
        }
    }
}

/// Executes an operation with panic capture.
///
/// Returns Ok(()) if the operation completes without invariant violation,
/// or Err(InvariantViolation) if a panic or invariant check fails.
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

/// Executes a single operation. Placeholder for now.
///
/// This will be implemented by the operation generators task (dr-epv.27.2).
fn execute_single_operation(
    _state: &mut ChaosMonkeyState,
    _op_type: OperationType,
) -> Result<(), LatticeError> {
    Ok(())
}

/// Creates a filter that matches all documents including closed ones.
fn all_documents_filter() -> DocumentFilter {
    DocumentFilter { include_closed: true, ..Default::default() }
}

/// Queries all documents from the index for an invariant check.
///
/// Creates a context and queries documents, converting any errors to
/// InvariantViolation with the specified invariant kind.
fn query_all_docs_for_check(
    state: &ChaosMonkeyState,
    invariant: InvariantKind,
) -> Result<Vec<DocumentRow>, InvariantViolation> {
    let context = state.create_context().map_err(|e| InvariantViolation {
        invariant,
        description: format!("Failed to create context: {e}"),
        affected_paths: vec![],
        affected_ids: vec![],
    })?;

    document_queries::query(&context.conn, &all_documents_filter()).map_err(|e| {
        InvariantViolation {
            invariant,
            description: format!("Failed to query index: {e}"),
            affected_paths: vec![],
            affected_ids: vec![],
        }
    })
}

/// Checks all invariants after an operation.
///
/// See appendix_chaos_monkey.md for the full list of invariants.
/// Note: Link path validity (invariant 9) is not yet implemented; it requires
/// parsing document content to verify links point to current paths after
/// close/reopen operations.
fn check_invariants(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    check_index_filesystem_consistency(state)?;
    check_id_uniqueness(state)?;
    check_id_format(state)?;
    check_closed_state_consistency(state)?;
    check_root_state_consistency(state)?;
    check_directory_state_consistency(state)?;
    // TODO: Add link path validity check (invariant 9 in appendix_chaos_monkey.md)
    Ok(())
}

/// Checks that every ID in the index has a corresponding file.
fn check_index_filesystem_consistency(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let context = state.create_context().map_err(|e| InvariantViolation {
        invariant: InvariantKind::IndexHasOrphanedId,
        description: format!("Failed to create context for invariant check: {e}"),
        affected_paths: vec![],
        affected_ids: vec![],
    })?;

    let index_docs =
        document_queries::query(&context.conn, &all_documents_filter()).map_err(|e| {
            InvariantViolation {
                invariant: InvariantKind::IndexHasOrphanedId,
                description: format!("Failed to query index: {e}"),
                affected_paths: vec![],
                affected_ids: vec![],
            }
        })?;

    for doc in &index_docs {
        let file_path = state.repo_root().join(&doc.path);
        if !file_path.exists() {
            return Err(InvariantViolation {
                invariant: InvariantKind::IndexHasOrphanedId,
                description: format!(
                    "Index contains ID {} at path {} but file does not exist",
                    doc.id, doc.path
                ),
                affected_paths: vec![file_path],
                affected_ids: vec![doc.id.clone()],
            });
        }
    }

    let filesystem_docs = find_markdown_files(state.repo_root())?;
    let indexed_paths: HashSet<_> = index_docs.iter().map(|d| PathBuf::from(&d.path)).collect();

    for file_path in &filesystem_docs {
        if let Ok(relative) = file_path.strip_prefix(state.repo_root()) {
            let relative_str = relative.to_string_lossy().to_string();
            if relative_str.contains(".lattice") || relative_str.contains(".git") {
                continue;
            }

            let doc_result = document_reader::read(file_path);
            if let Ok(doc) = doc_result
                && !indexed_paths.contains(relative)
            {
                return Err(InvariantViolation {
                    invariant: InvariantKind::FilesystemHasUnindexedDocument,
                    description: format!(
                        "Filesystem has document at {} with ID {} but it's not in index",
                        relative.display(),
                        doc.frontmatter.lattice_id
                    ),
                    affected_paths: vec![file_path.clone()],
                    affected_ids: vec![doc.frontmatter.lattice_id.to_string()],
                });
            }
        }
    }

    Ok(())
}

/// Checks that no two files share the same ID.
fn check_id_uniqueness(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let docs = query_all_docs_for_check(state, InvariantKind::DuplicateId)?;

    let mut id_to_path: HashMap<String, String> = HashMap::new();
    for doc in docs {
        if let Some(existing_path) = id_to_path.get(&doc.id) {
            return Err(InvariantViolation {
                invariant: InvariantKind::DuplicateId,
                description: format!(
                    "Duplicate ID {} found in {} and {}",
                    doc.id, existing_path, doc.path
                ),
                affected_paths: vec![PathBuf::from(existing_path), PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
        id_to_path.insert(doc.id.clone(), doc.path.clone());
    }

    Ok(())
}

/// Checks that all IDs in the index are valid Lattice IDs.
fn check_id_format(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let docs = query_all_docs_for_check(state, InvariantKind::MalformedIdInIndex)?;

    for doc in docs {
        if doc.id.parse::<LatticeId>().is_err() {
            return Err(InvariantViolation {
                invariant: InvariantKind::MalformedIdInIndex,
                description: format!(
                    "Index contains malformed ID '{}' at path {}",
                    doc.id, doc.path
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that is_closed in index matches path containing .closed/.
fn check_closed_state_consistency(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let docs = query_all_docs_for_check(state, InvariantKind::ClosedStateInconsistency)?;

    for doc in docs {
        let path_indicates_closed =
            doc.path.contains("/tasks/.closed/") || doc.path.contains("/.closed/");

        if doc.is_closed != path_indicates_closed {
            return Err(InvariantViolation {
                invariant: InvariantKind::ClosedStateInconsistency,
                description: format!(
                    "Document {} has is_closed={} but path '{}' {} .closed/",
                    doc.id,
                    doc.is_closed,
                    doc.path,
                    if path_indicates_closed { "contains" } else { "does not contain" }
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that is_root in index matches filename = directory name.
fn check_root_state_consistency(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let docs = query_all_docs_for_check(state, InvariantKind::RootStateInconsistency)?;

    for doc in docs {
        let path = PathBuf::from(&doc.path);
        let is_root_by_path = is_root_document(&path);

        if doc.is_root != is_root_by_path {
            return Err(InvariantViolation {
                invariant: InvariantKind::RootStateInconsistency,
                description: format!(
                    "Document {} has is_root={} but path '{}' {} a root document",
                    doc.id,
                    doc.is_root,
                    doc.path,
                    if is_root_by_path { "is" } else { "is not" }
                ),
                affected_paths: vec![path],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that in_tasks_dir and in_docs_dir match path components.
fn check_directory_state_consistency(state: &ChaosMonkeyState) -> Result<(), InvariantViolation> {
    let docs = query_all_docs_for_check(state, InvariantKind::DirectoryStateInconsistency)?;

    for doc in docs {
        let in_tasks = doc.path.contains("/tasks/");
        let in_docs = doc.path.contains("/docs/");

        if doc.in_tasks_dir != in_tasks {
            return Err(InvariantViolation {
                invariant: InvariantKind::DirectoryStateInconsistency,
                description: format!(
                    "Document {} has in_tasks_dir={} but path '{}'",
                    doc.id, doc.in_tasks_dir, doc.path
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }

        if doc.in_docs_dir != in_docs {
            return Err(InvariantViolation {
                invariant: InvariantKind::DirectoryStateInconsistency,
                description: format!(
                    "Document {} has in_docs_dir={} but path '{}'",
                    doc.id, doc.in_docs_dir, doc.path
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
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
    info!(repo_root = %repo_root.display(), "Initializing test repository");

    let output =
        Command::new("git").args(["init"]).current_dir(repo_root).output().map_err(|e| {
            LatticeError::GitError {
                operation: "init".to_string(),
                reason: format!("Failed to run git init: {e}"),
            }
        })?;

    if !output.status.success() {
        return Err(LatticeError::GitError {
            operation: "init".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

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

/// Configures git user for commits.
fn configure_git_user(repo_root: &Path) -> LatticeResult<()> {
    let config_cmds = [["config", "user.email", "chaosmonkey@lattice.test"], [
        "config",
        "user.name",
        "Chaos Monkey",
    ]];

    for args in &config_cmds {
        let output =
            Command::new("git").args(args).current_dir(repo_root).output().map_err(|e| {
                LatticeError::GitError {
                    operation: format!("config {}", args[1]),
                    reason: format!("Failed to run git config: {e}"),
                }
            })?;

        if !output.status.success() {
            return Err(LatticeError::GitError {
                operation: format!("config {}", args[1]),
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
    }

    Ok(())
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
    let add_output =
        Command::new("git").args(["add", "."]).current_dir(repo_root).output().map_err(|e| {
            LatticeError::GitError {
                operation: "add".to_string(),
                reason: format!("Failed to run git add: {e}"),
            }
        })?;

    if !add_output.status.success() {
        return Err(LatticeError::GitError {
            operation: "add".to_string(),
            reason: String::from_utf8_lossy(&add_output.stderr).to_string(),
        });
    }

    let commit_output = Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| LatticeError::GitError {
            operation: "commit".to_string(),
            reason: format!("Failed to run git commit: {e}"),
        })?;

    if !commit_output.status.success() {
        return Err(LatticeError::GitError {
            operation: "commit".to_string(),
            reason: String::from_utf8_lossy(&commit_output.stderr).to_string(),
        });
    }

    info!("Created initial commit");
    Ok(())
}

/// Finds all markdown files in a directory recursively.
fn find_markdown_files(dir: &Path) -> Result<Vec<PathBuf>, InvariantViolation> {
    let mut files = Vec::new();
    find_markdown_files_recursive(dir, &mut files)?;
    Ok(files)
}

fn find_markdown_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), InvariantViolation> {
    let entries = fs::read_dir(dir).map_err(|e| InvariantViolation {
        invariant: InvariantKind::FilesystemHasUnindexedDocument,
        description: format!("Failed to read directory {}: {e}", dir.display()),
        affected_paths: vec![dir.to_path_buf()],
        affected_ids: vec![],
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| InvariantViolation {
            invariant: InvariantKind::FilesystemHasUnindexedDocument,
            description: format!("Failed to read directory entry: {e}"),
            affected_paths: vec![dir.to_path_buf()],
            affected_ids: vec![],
        })?;

        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != ".git" && name != ".lattice" {
                find_markdown_files_recursive(&path, files)?;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }

    Ok(())
}

/// Checks if a path represents a root document.
fn is_root_document(path: &Path) -> bool {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent_name =
        path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("");

    let stem_without_underscore = file_stem.strip_prefix('_').unwrap_or(file_stem);

    stem_without_underscore == parent_name
}

/// Prints the chaos monkey result.
fn print_result(result: &ChaosMonkeyResult) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if result.success {
        println!("✅ Chaos Monkey completed successfully!");
        println!("   Operations: {}/{}", result.operations_completed, result.max_ops);
        println!("   Seed: {}", result.seed);
    } else {
        println!("❌ Chaos Monkey found an invariant violation!");
        println!();
        println!("   Seed: {}", result.seed);
        println!("   Operations completed: {}", result.operations_completed);

        if let Some(ref violation) = result.violation {
            println!();
            println!("   Invariant: {}", violation.invariant.name());
            println!("   Description: {}", violation.description);

            if !violation.affected_paths.is_empty() {
                println!("   Affected paths:");
                for path in &violation.affected_paths {
                    println!("     - {}", path.display());
                }
            }

            if !violation.affected_ids.is_empty() {
                println!("   Affected IDs:");
                for id in &violation.affected_ids {
                    println!("     - {id}");
                }
            }
        }

        if let Some(ref op) = result.failing_operation {
            println!();
            println!("   Failing operation #{}: {}", op.number, op.op_type.name());
            println!("   Description: {}", op.description);
            if let Some(ref err) = op.error_message {
                println!("   Error: {err}");
            }
        }

        println!();
        println!("   To reproduce, run:");
        println!("     lat chaosmonkey --seed {}", result.seed);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
