use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info};

use crate::claim::claim_operations;
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_filter::{SortColumn, SortOrder};
use crate::index::document_types;
use crate::index::document_types::DocumentRow;
use crate::task::task_priority::Priority;

/// Sort policy for ready task results.
///
/// Controls how ready tasks are ordered in query results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadySortPolicy {
    /// Balance priority and creation date (default).
    ///
    /// Primary sort by priority (P0 first), secondary by creation date (oldest
    /// first within same priority). This surfaces high-priority old work.
    #[default]
    Hybrid,

    /// Strict priority ordering.
    ///
    /// Sorts only by priority (P0 first), with no secondary sort. Ties are
    /// returned in database order.
    Priority,

    /// Creation date ordering.
    ///
    /// Sorts by creation date (oldest first), ignoring priority. Useful for
    /// processing work in FIFO order.
    Oldest,
}

/// Filter options for ready task queries.
///
/// By default, ready queries exclude P4 (backlog) and claimed tasks.
/// Use the builder methods to include them.
#[derive(Debug, Clone, Default)]
pub struct ReadyFilter {
    /// Include P4 (backlog) tasks in results. Default: `false`.
    pub include_backlog: bool,

    /// Include claimed tasks in results. Default: `false`.
    pub include_claimed: bool,

    /// Filter to descendants of this directory path.
    pub path_prefix: Option<String>,

    /// Filter by task type (bug, feature, task, chore).
    pub task_type: Option<TaskType>,

    /// Filter by exact priority level (0-4).
    pub priority: Option<u8>,

    /// Require ALL of these labels (AND semantics).
    pub labels_all: Vec<String>,

    /// Require ANY of these labels (OR semantics).
    pub labels_any: Vec<String>,

    /// Maximum number of results to return.
    pub limit: Option<u32>,

    /// Sort policy for results.
    pub sort_policy: ReadySortPolicy,
}

/// Result of a ready task query.
#[derive(Debug, Clone)]
pub struct ReadyTask {
    /// The document row from the index.
    pub document: DocumentRow,

    /// Whether this task is currently claimed (always false unless
    /// `include_claimed` was set).
    pub claimed: bool,
}

/// Queries tasks that are ready for work.
///
/// Returns tasks matching the ready criteria:
/// - Not closed (not in `.closed/` directory)
/// - Not blocked (all `blocked-by` tasks are closed)
/// - Priority is not P4 (unless `include_backlog` is set)
/// - Not claimed (unless `include_claimed` is set)
///
/// # Arguments
///
/// * `conn` - Database connection for querying the index
/// * `repo_root` - Repository root for claim checks
/// * `filter` - Filter and sort options
///
/// # Errors
///
/// Returns `LatticeError` if database queries or claim checks fail.
pub fn query_ready_tasks(
    conn: &Connection,
    repo_root: &Path,
    filter: &ReadyFilter,
) -> Result<Vec<ReadyTask>, LatticeError> {
    let (sql, params) = build_ready_query(filter);
    debug!(sql = sql.as_str(), "Executing ready tasks query");

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare ready query: {e}"),
    })?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();
    let rows = stmt
        .query_map(params_refs.as_slice(), document_types::row_to_document)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to execute ready query: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect ready query results: {e}"),
        })?;

    debug!(count = rows.len(), "Ready query returned results before claim filtering");

    let mut results = Vec::new();
    let limit = filter.limit.map(|l| l as usize);

    for document in rows {
        // Stop early if we've hit the limit
        if let Some(max) = limit
            && results.len() >= max
        {
            break;
        }

        let id = LatticeId::parse(&document.id)?;
        let claimed = claim_operations::is_claimed(repo_root, &id)?;

        if claimed && !filter.include_claimed {
            debug!(id = document.id.as_str(), "Excluding claimed task");
            continue;
        }

        results.push(ReadyTask { document, claimed });
    }

    info!(count = results.len(), "Returning ready tasks");
    Ok(results)
}

/// Counts ready tasks without fetching full documents.
///
/// Uses the same criteria as [`query_ready_tasks`] but returns only a count.
/// Does not check claims (claim filtering happens post-query).
pub fn count_ready_tasks(conn: &Connection, filter: &ReadyFilter) -> Result<u32, LatticeError> {
    let (sql, params) = build_ready_count_query(filter);
    debug!(sql = sql.as_str(), "Executing ready count query");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();
    let count: i64 =
        conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError {
                reason: format!("Failed to execute ready count query: {e}"),
            }
        })?;

    debug!(count, "Ready count query returned");
    Ok(count as u32)
}

/// Returns the sort column and order for a given policy.
///
/// Useful for integrating with [`DocumentFilter`] when needed.
pub fn sort_for_policy(policy: ReadySortPolicy) -> (SortColumn, SortOrder) {
    match policy {
        ReadySortPolicy::Hybrid | ReadySortPolicy::Priority => {
            (SortColumn::Priority, SortOrder::Ascending)
        }
        ReadySortPolicy::Oldest => (SortColumn::CreatedAt, SortOrder::Ascending),
    }
}

impl ReadyFilter {
    /// Creates a new filter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Includes P4 (backlog) tasks in results.
    pub fn with_include_backlog(mut self) -> Self {
        self.include_backlog = true;
        self
    }

    /// Includes claimed tasks in results.
    pub fn with_include_claimed(mut self) -> Self {
        self.include_claimed = true;
        self
    }

    /// Filters to descendants of the given path prefix.
    pub fn with_path_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(prefix.into());
        self
    }

    /// Filters by task type.
    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Filters by exact priority level (0-4).
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Requires all specified labels (AND semantics).
    pub fn with_labels_all(mut self, labels: Vec<String>) -> Self {
        self.labels_all = labels;
        self
    }

    /// Requires any of the specified labels (OR semantics).
    pub fn with_labels_any(mut self, labels: Vec<String>) -> Self {
        self.labels_any = labels;
        self
    }

    /// Limits the number of results.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the sort policy.
    pub fn with_sort_policy(mut self, policy: ReadySortPolicy) -> Self {
        self.sort_policy = policy;
        self
    }
}

fn build_ready_query(filter: &ReadyFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("SELECT * FROM documents WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    append_ready_conditions(&mut sql, &mut params, filter);
    append_ready_sort(&mut sql, filter);

    // Only apply SQL-level limit if we're including claimed tasks
    // (claim filtering happens after SQL, so limit must be applied afterward when
    // filtering)
    if filter.include_claimed
        && let Some(limit) = filter.limit
    {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }

    (sql, params)
}

fn build_ready_count_query(filter: &ReadyFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("SELECT COUNT(*) FROM documents WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    append_ready_conditions(&mut sql, &mut params, filter);

    (sql, params)
}

fn append_ready_conditions(
    sql: &mut String,
    params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    filter: &ReadyFilter,
) {
    // Must be a task (has task_type)
    sql.push_str(" AND task_type IS NOT NULL");

    // Not closed
    sql.push_str(" AND is_closed = 0");

    // Not blocked: check both blocked_by links (this doc is source) and
    // blocking links (this doc is target) to find all open blockers
    sql.push_str(
        " AND NOT EXISTS (
            SELECT 1 FROM links l
            JOIN documents d2 ON l.target_id = d2.id
            WHERE l.source_id = documents.id
            AND l.link_type = 'blocked_by'
            AND d2.is_closed = 0
        )
        AND NOT EXISTS (
            SELECT 1 FROM links l
            JOIN documents d2 ON l.source_id = d2.id
            WHERE l.target_id = documents.id
            AND l.link_type = 'blocking'
            AND d2.is_closed = 0
        )",
    );

    // Priority filter (exclude P4 unless include_backlog)
    if !filter.include_backlog {
        sql.push_str(" AND priority < ?");
        params.push(Box::new(Priority::P4.as_u8() as i32));
    }

    // Optional path prefix filter
    if let Some(prefix) = &filter.path_prefix {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    // Optional task type filter
    if let Some(tt) = filter.task_type {
        sql.push_str(" AND task_type = ?");
        params.push(Box::new(tt.to_string()));
    }

    // Optional exact priority filter
    if let Some(priority) = filter.priority {
        sql.push_str(" AND priority = ?");
        params.push(Box::new(priority as i32));
    }

    // Label filters (AND semantics)
    for label in &filter.labels_all {
        sql.push_str(
            " AND EXISTS (SELECT 1 FROM labels WHERE labels.document_id = documents.id AND labels.label = ?)",
        );
        params.push(Box::new(label.clone()));
    }

    // Label filters (OR semantics)
    if !filter.labels_any.is_empty() {
        let placeholders: Vec<&str> = filter.labels_any.iter().map(|_| "?").collect();
        sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM labels WHERE labels.document_id = documents.id AND labels.label IN ({}))",
            placeholders.join(", ")
        ));
        for label in &filter.labels_any {
            params.push(Box::new(label.clone()));
        }
    }
}

fn append_ready_sort(sql: &mut String, filter: &ReadyFilter) {
    match filter.sort_policy {
        ReadySortPolicy::Hybrid => {
            // Priority first (ascending = P0 first), then created_at (ascending = oldest
            // first)
            sql.push_str(" ORDER BY priority ASC, created_at ASC");
        }
        ReadySortPolicy::Priority => {
            sql.push_str(" ORDER BY priority ASC");
        }
        ReadySortPolicy::Oldest => {
            sql.push_str(" ORDER BY created_at ASC");
        }
    }
}
