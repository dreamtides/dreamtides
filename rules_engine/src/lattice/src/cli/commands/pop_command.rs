use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info};

use crate::claim::claim_operations;
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::show_command::document_formatter::{self, OutputMode, ShowOutput};
use crate::cli::commands::show_command::show_executor;
use crate::cli::shared_options::ReadySortPolicy as CliSortPolicy;
use crate::cli::workflow_args::PopArgs;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::{document_queries, view_tracking};
use crate::task::ready_calculator::{self, ReadyFilter, ReadySortPolicy};

/// JSON output for the pop command.
///
/// Wraps ShowOutput with additional metadata about the pop operation.
#[derive(Debug, Clone, Serialize)]
pub struct PopOutput {
    /// Whether this was a dry run (task not actually claimed).
    pub dry_run: bool,

    /// Whether claiming was skipped via --no-claim.
    pub no_claim: bool,

    /// The claimed task with full context.
    pub task: ShowOutput,
}

/// Executes the `lat pop` command.
///
/// Finds the highest-priority ready task, claims it, and outputs full context.
/// This is the primary interface for AI agents to start work on tasks.
pub fn execute(context: CommandContext, args: PopArgs) -> LatticeResult<()> {
    info!(dry_run = args.dry_run, no_claim = args.no_claim, "Executing pop command");

    let filter = build_filter(&context, &args)?;
    let tasks = ready_calculator::query_ready_tasks(&context.conn, &context.repo_root, &filter)?;

    if tasks.is_empty() {
        return handle_no_tasks(&context, &args);
    }

    let task = &tasks[0];
    debug!(id = task.document.id.as_str(), "Selected highest-priority ready task");

    let should_claim = !args.dry_run && !args.no_claim;
    if should_claim {
        if let Some(max_claims) = args.max_claims {
            let current_claims = claim_operations::list_claims(&context.repo_root)?.len();
            if current_claims >= max_claims {
                return Err(LatticeError::ClaimLimitExceeded {
                    current: current_claims,
                    max: max_claims,
                });
            }
        }
        let id = LatticeId::parse(&task.document.id)?;
        claim_operations::claim_task(&context.repo_root, &id, &context.repo_root)?;
        info!(id = task.document.id.as_str(), "Claimed task");
    }

    view_tracking::record_view(&context.conn, &task.document.id)?;

    let show_output = show_executor::build_full_output(&context, &task.document)?;

    output_result(&context, &args, show_output)?;

    Ok(())
}

/// Builds a ReadyFilter from PopArgs.
fn build_filter(context: &CommandContext, args: &PopArgs) -> LatticeResult<ReadyFilter> {
    let mut filter = ReadyFilter::new();

    if args.include_backlog {
        filter = filter.with_include_backlog();
        debug!("Including backlog (P4) tasks");
    }

    filter = filter.with_limit(1);
    filter = filter.with_sort_policy(convert_sort_policy(args.sort));
    debug!(sort = ?args.sort, "Using sort policy");

    if let Some(parent_id) = &args.filter.parent {
        let path_prefix = resolve_parent_to_path(&context.conn, parent_id)?;
        filter = filter.with_path_prefix(path_prefix);
    }

    if let Some(path) = &args.filter.path {
        filter = filter.with_path_prefix(path.clone());
        debug!(path = path.as_str(), "Filtering by path prefix");
    }

    if let Some(task_type) = args.filter.r#type {
        filter = filter.with_task_type(task_type);
        debug!(task_type = ?task_type, "Filtering by task type");
    }

    if let Some(priority) = args.filter.priority {
        filter = filter.with_priority(priority);
        debug!(priority, "Filtering by exact priority");
    }

    if !args.filter.label.is_empty() {
        filter = filter.with_labels_all(args.filter.label.clone());
        debug!(labels = ?args.filter.label, "Filtering by labels (AND)");
    }

    if !args.filter.label_any.is_empty() {
        filter = filter.with_labels_any(args.filter.label_any.clone());
        debug!(labels = ?args.filter.label_any, "Filtering by labels (OR)");
    }

    Ok(filter)
}

/// Resolves a parent ID to a path prefix for filtering.
fn resolve_parent_to_path(conn: &Connection, parent_id: &str) -> LatticeResult<String> {
    let doc = document_queries::lookup_by_id(conn, parent_id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: parent_id.to_string() })?;

    let path_prefix = std::path::Path::new(&doc.path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    debug!(
        parent_id = parent_id,
        path_prefix = path_prefix.as_str(),
        "Resolved parent ID to path prefix"
    );

    if doc.is_root {
        let dir_path = std::path::Path::new(&doc.path)
            .parent()
            .map(|p| {
                let s = p.to_string_lossy().to_string();
                if s.is_empty() { ".".to_string() } else { s }
            })
            .unwrap_or_else(|| ".".to_string());
        debug!(dir_path = dir_path.as_str(), "Using root document directory");
        return Ok(dir_path);
    }

    Ok(path_prefix)
}

/// Converts CLI sort policy to ready_calculator sort policy.
fn convert_sort_policy(cli_policy: CliSortPolicy) -> ReadySortPolicy {
    match cli_policy {
        CliSortPolicy::Hybrid => ReadySortPolicy::Hybrid,
        CliSortPolicy::Priority => ReadySortPolicy::Priority,
        CliSortPolicy::Oldest => ReadySortPolicy::Oldest,
    }
}

/// Handles the case when no ready tasks are available.
fn handle_no_tasks(_context: &CommandContext, _args: &PopArgs) -> LatticeResult<()> {
    Ok(())
}

/// Outputs the pop result in the appropriate format.
fn output_result(
    context: &CommandContext,
    args: &PopArgs,
    show_output: ShowOutput,
) -> LatticeResult<()> {
    if context.global.json {
        let output =
            PopOutput { dry_run: args.dry_run, no_claim: args.no_claim, task: show_output };
        let json_str =
            serde_json::to_string_pretty(&output).expect("PopOutput serialization should not fail");
        println!("{json_str}");
    } else {
        if args.dry_run {
            println!("(dry run - task not claimed)");
            println!();
        } else if args.no_claim {
            println!("(--no-claim - task not claimed)");
            println!();
        } else {
            println!("Claimed: {}", show_output.id);
            println!();
        }

        let mode = if args.raw { OutputMode::Raw } else { OutputMode::Full };
        document_formatter::print_output(&show_output, mode, false);
    }

    Ok(())
}
