use rusqlite::Connection;
use tracing::debug;

use crate::cli::shared_options::ReadySortPolicy as CliSortPolicy;
use crate::cli::workflow_args::ReadyArgs;
use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::task::ready_calculator::{ReadyFilter, ReadySortPolicy};

/// Builds a `ReadyFilter` from CLI arguments.
///
/// Handles translation of CLI-level options (like `--parent <id>`) into
/// filter-level options (like `path_prefix`). The `--parent` flag accepts a
/// Lattice ID and resolves it to a path prefix via the index.
pub fn build_filter(conn: &Connection, args: &ReadyArgs) -> Result<ReadyFilter, LatticeError> {
    let mut filter = ReadyFilter::new();

    // Handle --include-backlog
    if args.include_backlog {
        filter = filter.with_include_backlog();
        debug!("Including backlog (P4) tasks");
    }

    // Handle --include-claimed
    if args.include_claimed {
        filter = filter.with_include_claimed();
        debug!("Including claimed tasks");
    }

    // Handle --discrete
    if args.discrete {
        filter = filter.with_discrete();
        debug!("Discrete mode: excluding directories with existing claims");
    }

    // Handle --limit
    if let Some(limit) = args.limit {
        filter = filter.with_limit(limit as u32);
        debug!(limit, "Applying limit");
    }

    // Handle sort policy
    filter = filter.with_sort_policy(convert_sort_policy(args.sort));
    debug!(sort = ?args.sort, "Using sort policy");

    // Handle filter options
    filter = apply_filter_options(conn, filter, args)?;

    Ok(filter)
}

/// Applies filter options from ReadyArgs to the ReadyFilter.
fn apply_filter_options(
    conn: &Connection,
    mut filter: ReadyFilter,
    args: &ReadyArgs,
) -> Result<ReadyFilter, LatticeError> {
    // Handle --parent (resolve ID to path prefix)
    if let Some(parent_id) = &args.filter.parent {
        let path_prefix = resolve_parent_to_path(conn, parent_id)?;
        filter = filter.with_path_prefix(path_prefix);
    }

    // Handle --path (direct path prefix)
    if let Some(path) = &args.filter.path {
        filter = filter.with_path_prefix(path.clone());
        debug!(path = path.as_str(), "Filtering by path prefix");
    }

    // Handle --type
    if let Some(task_type) = args.filter.r#type {
        filter = filter.with_task_type(task_type);
        debug!(task_type = ?task_type, "Filtering by task type");
    }

    // Handle --priority (exact match)
    if let Some(priority) = args.filter.priority {
        filter = filter.with_priority(priority);
        debug!(priority, "Filtering by exact priority");
    }

    // Handle --label (AND semantics)
    if !args.filter.label.is_empty() {
        filter = filter.with_labels_all(args.filter.label.clone());
        debug!(labels = ?args.filter.label, "Filtering by labels (AND)");
    }

    // Handle --label-any (OR semantics)
    if !args.filter.label_any.is_empty() {
        filter = filter.with_labels_any(args.filter.label_any.clone());
        debug!(labels = ?args.filter.label_any, "Filtering by labels (OR)");
    }

    Ok(filter)
}

/// Resolves a parent ID to a path prefix for filtering.
///
/// Looks up the document by ID and returns its directory path. Returns an error
/// if the document is not found.
fn resolve_parent_to_path(conn: &Connection, parent_id: &str) -> Result<String, LatticeError> {
    let doc = document_queries::lookup_by_id(conn, parent_id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: parent_id.to_string() })?;

    // Extract directory from path
    let path_prefix = std::path::Path::new(&doc.path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    debug!(
        parent_id = parent_id,
        path_prefix = path_prefix.as_str(),
        "Resolved parent ID to path prefix"
    );

    // If the document is a root document (filename matches directory), use the
    // directory path
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
