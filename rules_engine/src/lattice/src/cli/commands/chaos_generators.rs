//! Operation generators for chaos monkey fuzz testing.
//!
//! This module provides random operation generators for the chaos monkey
//! command. Each generator produces operations that test different aspects
//! of Lattice's robustness.

use std::fs;
use std::process::Command;

use chrono::Utc;
use tracing::{debug, info, warn};

use crate::cli::commands::chaos_monkey::{ChaosMonkeyState, OperationType};
use crate::cli::commands::check_command::check_executor;
use crate::cli::commands::{
    close_command, create_command, dep_command, mv_command, prune_command, reopen_command,
    search_command, update_command,
};
use crate::cli::maintenance_args::CheckArgs;
use crate::cli::query_args::SearchArgs;
use crate::cli::structure_args::{DepArgs, DepCommand};
use crate::cli::task_args::{CloseArgs, CreateArgs, MvArgs, PruneArgs, ReopenArgs, UpdateArgs};
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;

/// Word lists for generating random descriptions.
const ADJECTIVES: &[&str] = &[
    "fast", "slow", "broken", "fixed", "new", "old", "critical", "minor", "urgent", "pending",
    "complete", "partial", "simple", "complex", "easy", "hard", "large", "small",
];
const NOUNS: &[&str] = &[
    "bug", "feature", "task", "issue", "error", "function", "module", "test", "doc", "config",
    "api", "endpoint", "database", "query", "cache", "service", "handler", "parser",
];
const VERBS: &[&str] = &[
    "fix",
    "add",
    "remove",
    "update",
    "refactor",
    "test",
    "document",
    "optimize",
    "implement",
    "review",
    "deploy",
    "configure",
    "migrate",
    "validate",
    "cleanup",
];

/// Maximum number of labels to add to a document.
const MAX_LABELS: usize = 3;

/// Dispatches to the appropriate generator based on operation type.
pub fn execute_operation(
    state: &mut ChaosMonkeyState,
    op_type: OperationType,
) -> Result<(), LatticeError> {
    match op_type {
        OperationType::Create => generate_create(state),
        OperationType::Update => generate_update(state),
        OperationType::Close => generate_close(state),
        OperationType::Reopen => generate_reopen(state),
        OperationType::Prune => generate_prune(state),
        OperationType::Move => generate_move(state),
        OperationType::Search => generate_search(state),
        OperationType::RebuildIndex => generate_rebuild_index(state),
        OperationType::FilesystemCreate => generate_filesystem_create(state),
        OperationType::FilesystemDelete => generate_filesystem_delete(state),
        OperationType::FilesystemModify => generate_filesystem_modify(state),
        OperationType::GitCommit => generate_git_commit(state),
        OperationType::GitCheckout => generate_git_checkout(state),
        OperationType::DepAdd => generate_dep_add(state),
        OperationType::DepRemove => generate_dep_remove(state),
    }
}

/// Generates a random description for a document.
fn random_description(state: &mut ChaosMonkeyState) -> String {
    let verb = VERBS[state.random_range(0, VERBS.len())];
    let adj = ADJECTIVES[state.random_range(0, ADJECTIVES.len())];
    let noun = NOUNS[state.random_range(0, NOUNS.len())];
    let mut chars = verb.chars();
    let capitalized: String = match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    };
    format!("{} {} {}", capitalized, adj, noun)
}

/// Generates random labels.
fn random_labels(state: &mut ChaosMonkeyState) -> Vec<String> {
    let count = state.random_range(0, MAX_LABELS + 1);
    let label_pool = ["frontend", "backend", "urgent", "low-priority", "tech-debt", "security"];
    (0..count).map(|_| label_pool[state.random_range(0, label_pool.len())].to_string()).collect()
}

/// Generates a random task type.
fn random_task_type(state: &mut ChaosMonkeyState) -> TaskType {
    match state.random_range(0, 4) {
        0 => TaskType::Bug,
        1 => TaskType::Feature,
        2 => TaskType::Task,
        _ => TaskType::Chore,
    }
}

/// Generates a `lat create` operation.
fn generate_create(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let is_task = state.random_bool();

    let args = CreateArgs {
        parent: Some("project".to_string()),
        description: Some(random_description(state)),
        r#type: if is_task { Some(random_task_type(state)) } else { None },
        priority: if is_task { Some(state.random_range(0, 5) as u8) } else { None },
        body_file: None,
        labels: random_labels(state),
        deps: None,
        interactive: false,
    };

    info!(
        description = ?args.description,
        is_task,
        priority = ?args.priority,
        "Generating create operation"
    );

    create_command::execute(context, args)
}

/// Generates a `lat update` operation on a random existing document.
fn generate_update(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let docs = document_queries::query(&context.conn, &DocumentFilter::default())?;

    if docs.is_empty() {
        debug!("No documents to update, skipping");
        return Ok(());
    }

    let doc = &docs[state.random_range(0, docs.len())];

    let mut args = UpdateArgs {
        ids: vec![doc.id.clone()],
        priority: None,
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };

    // Randomly decide what to update
    let update_kind = state.random_range(0, 4);
    match update_kind {
        0 if doc.task_type.is_some() => {
            args.priority = Some(state.random_range(0, 5) as u8);
        }
        1 if doc.task_type.is_some() => {
            args.r#type = Some(random_task_type(state));
        }
        2 => {
            args.add_labels = random_labels(state);
        }
        _ => {
            args.remove_labels = random_labels(state);
        }
    }

    info!(id = doc.id, update_kind, "Generating update operation");

    update_command::execute(context, args)
}

/// Generates a `lat close` operation on a random open task.
fn generate_close(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;

    let filter = DocumentFilter { include_closed: false, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;
    let tasks: Vec<_> = docs.iter().filter(|d| d.task_type.is_some() && !d.is_closed).collect();

    if tasks.is_empty() {
        debug!("No open tasks to close, skipping");
        return Ok(());
    }

    let task = tasks[state.random_range(0, tasks.len())];
    let reason = if state.random_bool() {
        Some(format!("Chaos monkey closure #{}", state.operations_completed()))
    } else {
        None
    };

    let args = CloseArgs { ids: vec![task.id.clone()], reason, dry_run: false };

    info!(id = task.id, "Generating close operation");

    close_command::execute(context, args)
}

/// Generates a `lat reopen` operation on a random closed task.
fn generate_reopen(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;

    let filter = DocumentFilter { include_closed: true, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;
    let closed_tasks: Vec<_> =
        docs.iter().filter(|d| d.task_type.is_some() && d.is_closed).collect();

    if closed_tasks.is_empty() {
        debug!("No closed tasks to reopen, skipping");
        return Ok(());
    }

    let task = closed_tasks[state.random_range(0, closed_tasks.len())];
    let args = ReopenArgs { ids: vec![task.id.clone()], dry_run: false };

    info!(id = task.id, "Generating reopen operation");

    reopen_command::execute(context, args)
}

/// Generates a `lat prune` operation.
fn generate_prune(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let use_force = state.random_bool();

    let args = PruneArgs { path: None, all: true, force: use_force, dry_run: false };

    info!(use_force, "Generating prune operation");

    prune_command::execute(context, args)
}

/// Generates a `lat mv` operation.
fn generate_move(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let docs = document_queries::query(&context.conn, &DocumentFilter::default())?;
    let movable: Vec<_> = docs.iter().filter(|d| !d.is_root).collect();

    if movable.is_empty() {
        debug!("No movable documents, skipping");
        return Ok(());
    }

    let doc = movable[state.random_range(0, movable.len())];

    // Generate a new path - sometimes valid, sometimes invalid
    let new_path = if state.random_bool() {
        // Valid path in project directory
        let subdir = if doc.task_type.is_some() { "tasks" } else { "docs" };
        let new_name = format!("moved_{}.md", state.random_range(1, 1000));
        format!("project/{}/{}", subdir, new_name)
    } else {
        // Potentially invalid path
        format!("invalid_path_{}.md", state.random_range(1, 100))
    };

    let args = MvArgs { id: doc.id.clone(), new_path: new_path.clone(), dry_run: false };

    info!(id = doc.id, new_path, "Generating move operation");

    mv_command::execute(context, args)
}

/// Generates a `lat search` operation.
fn generate_search(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;

    let queries = ["fix", "bug", "feature", "test", "project", "random_nonexistent_query_xyz"];
    let query = queries[state.random_range(0, queries.len())].to_string();

    let args = SearchArgs {
        query: query.clone(),
        limit: Some(state.random_range(1, 50)),
        path: None,
        r#type: None,
    };

    info!(query, "Generating search operation");

    search_command::execute(context, args)
}

/// Generates a `lat check --rebuild-index` operation.
fn generate_rebuild_index(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;

    let args = CheckArgs {
        path: None,
        errors_only: false,
        fix: false,
        staged_only: false,
        rebuild_index: true,
    };

    info!("Generating rebuild-index operation");

    check_executor::execute(context, args)
}

/// Generates a filesystem create operation (bypassing lat).
fn generate_filesystem_create(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let create_type = state.random_range(0, 3);

    match create_type {
        0 => create_valid_lattice_file(state),
        1 => create_invalid_lattice_file(state),
        _ => create_non_lattice_file(state),
    }
}

/// Creates a file with structurally valid Lattice frontmatter.
///
/// The file has all required YAML fields, but the generated ID (e.g.,
/// `LCHAOS1234`) may not conform to the strict Lattice ID format (Base32 A-Z,
/// 2-7). This tests how the system handles files that look valid but have
/// invalid IDs.
fn create_valid_lattice_file(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let filename = format!("chaos_valid_{}.md", state.random_range(1, 10000));
    let file_path = state.repo_root().join("project/tasks").join(&filename);

    // Generate a test ID that has valid structure but may use invalid Base32
    // characters. This tests handling of files with plausible-looking but
    // malformed IDs.
    let id_suffix = state.random_range(1000, 9999);
    let id = format!("LCHAOS{}", id_suffix);

    let content = format!(
        r#"---
lattice-id: {}
name: {}
description: Chaos monkey generated file
task-type: task
priority: 2
created-at: {}
updated-at: {}
---

# Chaos Monkey File

This file was created by chaos monkey.
"#,
        id,
        filename.trim_end_matches(".md"),
        Utc::now().to_rfc3339(),
        Utc::now().to_rfc3339()
    );

    info!(path = %file_path.display(), id, "Creating valid Lattice file via filesystem");

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: e.to_string(),
        })?;
    }

    fs::write(&file_path, content)
        .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })
}

/// Creates a file with invalid Lattice headers.
fn create_invalid_lattice_file(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let filename = format!("chaos_invalid_{}.md", state.random_range(1, 10000));
    let file_path = state.repo_root().join("project/tasks").join(&filename);

    let invalid_type = state.random_range(0, 4);
    let content = match invalid_type {
        0 => {
            // Missing required field
            "---\nlattice-id: LINVALID\nname: test\n---\nNo description field\n".to_string()
        }
        1 => {
            // Invalid ID format
            "---\nlattice-id: invalid-id\nname: test\ndescription: test\n---\nBad ID\n".to_string()
        }
        2 => {
            // Malformed YAML
            "---\nlattice-id: LMALFORM\nname: [broken yaml\n---\nBroken\n".to_string()
        }
        _ => {
            // Invalid priority
            "---\nlattice-id: LBADPRI\nname: test\ndescription: test\npriority: 99\n---\n"
                .to_string()
        }
    };

    info!(path = %file_path.display(), invalid_type, "Creating invalid Lattice file via filesystem");

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: e.to_string(),
        })?;
    }

    fs::write(&file_path, content)
        .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })
}

/// Creates a non-Lattice markdown file.
fn create_non_lattice_file(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let filename = format!("plain_{}.md", state.random_range(1, 10000));
    let file_path = state.repo_root().join("project").join(&filename);

    let content = "# Plain Markdown\n\nThis file has no Lattice frontmatter.\n";

    info!(path = %file_path.display(), "Creating non-Lattice markdown file");

    fs::write(&file_path, content)
        .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })
}

/// Generates a filesystem delete operation (bypassing lat).
fn generate_filesystem_delete(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let filter = DocumentFilter { include_closed: true, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;

    // Only delete non-root documents
    let deletable: Vec<_> = docs.iter().filter(|d| !d.is_root).collect();

    if deletable.is_empty() {
        debug!("No deletable documents, skipping filesystem delete");
        return Ok(());
    }

    let doc = deletable[state.random_range(0, deletable.len())];
    let file_path = state.repo_root().join(&doc.path);

    info!(path = %file_path.display(), id = doc.id, "Deleting file via filesystem");

    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })?;
    }

    Ok(())
}

/// Generates a filesystem modify operation (bypassing lat).
fn generate_filesystem_modify(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let filter = DocumentFilter { include_closed: true, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;

    if docs.is_empty() {
        debug!("No documents to modify, skipping");
        return Ok(());
    }

    let doc = &docs[state.random_range(0, docs.len())];
    let file_path = state.repo_root().join(&doc.path);

    if !file_path.exists() {
        debug!(path = %file_path.display(), "File doesn't exist, skipping modify");
        return Ok(());
    }

    let modify_type = state.random_range(0, 3);

    match modify_type {
        0 => {
            // Append to body
            let current = fs::read_to_string(&file_path).map_err(|e| LatticeError::ReadError {
                path: file_path.clone(),
                reason: e.to_string(),
            })?;
            let modified = format!(
                "{}\n\n<!-- Chaos monkey was here #{} -->\n",
                current,
                state.operations_completed()
            );
            info!(path = %file_path.display(), "Appending to file body");
            fs::write(&file_path, modified)
                .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })?;
        }
        1 => {
            // Corrupt the header slightly
            let current = fs::read_to_string(&file_path).map_err(|e| LatticeError::ReadError {
                path: file_path.clone(),
                reason: e.to_string(),
            })?;
            let modified = current.replace("updated-at:", "# corrupted updated-at:");
            info!(path = %file_path.display(), "Corrupting file header");
            fs::write(&file_path, modified)
                .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })?;
        }
        _ => {
            // Just touch the file (update mtime)
            let current = fs::read_to_string(&file_path).map_err(|e| LatticeError::ReadError {
                path: file_path.clone(),
                reason: e.to_string(),
            })?;
            info!(path = %file_path.display(), "Touching file");
            fs::write(&file_path, current)
                .map_err(|e| LatticeError::WriteError { path: file_path, reason: e.to_string() })?;
        }
    }

    Ok(())
}

/// Generates a git commit operation.
fn generate_git_commit(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let repo_root = state.repo_root();

    // Stage all changes
    let add_output =
        Command::new("git").args(["add", "-A"]).current_dir(repo_root).output().map_err(|e| {
            LatticeError::GitError { operation: "add".to_string(), reason: e.to_string() }
        })?;

    if !add_output.status.success() {
        warn!(stderr = %String::from_utf8_lossy(&add_output.stderr), "git add failed");
        return Ok(());
    }

    // Check if there are changes to commit
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| LatticeError::GitError {
            operation: "status".to_string(),
            reason: e.to_string(),
        })?;

    if status_output.stdout.is_empty() {
        debug!("No changes to commit");
        return Ok(());
    }

    let message = format!("Chaos monkey commit #{}", state.operations_completed());
    let commit_output = Command::new("git")
        .args(["commit", "-m", &message])
        .current_dir(repo_root)
        .output()
        .map_err(|e| LatticeError::GitError {
            operation: "commit".to_string(),
            reason: e.to_string(),
        })?;

    if !commit_output.status.success() {
        warn!(stderr = %String::from_utf8_lossy(&commit_output.stderr), "git commit failed");
    } else {
        info!(message, "Git commit created");
    }

    Ok(())
}

/// Generates a git checkout operation.
fn generate_git_checkout(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let branch_num = state.random_range(1, 10);
    let repo_root = state.repo_root().clone();
    let branch_name = format!("chaos-branch-{}", branch_num);

    // Try to create or switch to a branch
    let checkout_output = Command::new("git")
        .args(["checkout", "-B", &branch_name])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| LatticeError::GitError {
            operation: "checkout".to_string(),
            reason: e.to_string(),
        })?;

    if checkout_output.status.success() {
        info!(branch = branch_name, "Switched to branch");
    } else {
        warn!(
            stderr = %String::from_utf8_lossy(&checkout_output.stderr),
            branch = branch_name,
            "git checkout failed"
        );
    }

    Ok(())
}

/// Generates a `lat dep add` operation.
fn generate_dep_add(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let filter = DocumentFilter { include_closed: false, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;
    let tasks: Vec<_> = docs.iter().filter(|d| d.task_type.is_some()).collect();

    if tasks.len() < 2 {
        debug!("Not enough tasks for dependency, skipping");
        return Ok(());
    }

    let source_idx = state.random_range(0, tasks.len());
    let mut target_idx = state.random_range(0, tasks.len());
    while target_idx == source_idx {
        target_idx = state.random_range(0, tasks.len());
    }

    let source = &tasks[source_idx];
    let target = &tasks[target_idx];

    let args = DepArgs {
        command: DepCommand::Add { id: source.id.clone(), depends_on: target.id.clone() },
    };

    info!(source = source.id, target = target.id, "Generating dep add operation");

    dep_command::execute(context, args)
}

/// Generates a `lat dep remove` operation.
fn generate_dep_remove(state: &mut ChaosMonkeyState) -> Result<(), LatticeError> {
    let context = state.create_context()?;
    let filter = DocumentFilter { include_closed: false, ..Default::default() };
    let docs = document_queries::query(&context.conn, &filter)?;
    let tasks: Vec<_> = docs.iter().filter(|d| d.task_type.is_some()).collect();

    if tasks.len() < 2 {
        debug!("Not enough tasks for dependency removal, skipping");
        return Ok(());
    }

    // Pick random tasks - the remove will fail if no dependency exists, which is
    // fine
    let source_idx = state.random_range(0, tasks.len());
    let mut target_idx = state.random_range(0, tasks.len());
    while target_idx == source_idx {
        target_idx = state.random_range(0, tasks.len());
    }

    let source = &tasks[source_idx];
    let target = &tasks[target_idx];

    let args = DepArgs {
        command: DepCommand::Remove {
            id: source.id.clone(),
            depends_on: target.id.clone(),
            json: false,
        },
    };

    info!(source = source.id, target = target.id, "Generating dep remove operation");

    dep_command::execute(context, args)
}
