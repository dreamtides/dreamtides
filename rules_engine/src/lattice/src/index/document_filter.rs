//! Query builder for document filtering and sorting.
//!
//! Provides [`DocumentFilter`] using a builder pattern for SQL query
//! construction.
//!
//! # Example
//!
//! ```ignore
//! let filter = DocumentFilter::new()
//!     .with_state(DocumentState::Open)
//!     .with_priority_range(0, 2)
//!     .with_labels_all(vec!["urgent".into()])
//!     .sort_by(SortColumn::Priority)
//!     .limit(10);
//! ```
//!
//! # Available Filters
//!
//! - **State**: open, blocked, closed (via [`DocumentFilter::with_state`])
//! - **Priority**: exact or range (P0-P4)
//! - **Type**: bug, feature, task, chore
//! - **Labels**: AND semantics (`labels_all`) or OR semantics (`labels_any`)
//! - **Path/Name**: prefix matching, substring search (`name_contains`)
//! - **Directory**: root status, tasks/ or docs/ membership
//! - **Timestamps**: created/updated/closed with before/after bounds
//!
//! # Sorting
//!
//! Use [`SortColumn`] (UpdatedAt, CreatedAt, Priority, Name, Path, ViewCount)
//! and [`SortOrder`] (Ascending, Descending). Default: UpdatedAt descending.
//!
//! # Pagination
//!
//! Use `limit()` and `offset()` for paginated results.

use chrono::{DateTime, Utc};

use crate::document::frontmatter_schema::TaskType;

/// Filter criteria for querying documents from the index.
///
/// Uses a builder pattern for fluent configuration. By default, excludes
/// closed documents; use [`DocumentFilter::including_closed`] to include them.
#[derive(Debug, Clone, Default)]
pub struct DocumentFilter {
    /// Whether to include closed documents in results. Default: `false`.
    pub include_closed: bool,

    /// Filter by document state (open, blocked, or closed).
    pub state: Option<DocumentState>,

    /// Filter by exact priority value (0-4, where 0 is highest priority).
    pub priority: Option<u8>,

    /// Filter by priority range (min, max), inclusive on both ends.
    pub priority_range: Option<(u8, u8)>,

    /// Filter by task type (bug, feature, task, chore).
    pub task_type: Option<TaskType>,

    /// Require ALL of these labels (AND semantics).
    /// Document must have every label in this list.
    pub labels_all: Vec<String>,

    /// Require ANY of these labels (OR semantics).
    /// Document must have at least one label from this list.
    pub labels_any: Vec<String>,

    /// Filter by path prefix. Documents whose path starts with this string
    /// are included.
    pub path_prefix: Option<String>,

    /// Filter by name substring. Documents whose name contains this
    /// substring (case-sensitive) are included.
    pub name_contains: Option<String>,

    /// Include only documents created at or after this timestamp.
    pub created_after: Option<DateTime<Utc>>,

    /// Include only documents created at or before this timestamp.
    pub created_before: Option<DateTime<Utc>>,

    /// Include only documents updated at or after this timestamp.
    pub updated_after: Option<DateTime<Utc>>,

    /// Include only documents updated at or before this timestamp.
    pub updated_before: Option<DateTime<Utc>>,

    /// Include only documents closed at or after this timestamp.
    pub closed_after: Option<DateTime<Utc>>,

    /// Include only documents closed at or before this timestamp.
    pub closed_before: Option<DateTime<Utc>>,

    /// Filter to tasks discovered from a specific parent document.
    /// Returns documents that have the specified ID in their discovered-from
    /// frontmatter field.
    pub discovered_from: Option<String>,

    /// Filter by root document status. Root documents have a filename
    /// matching their containing directory (e.g., `api/api.md`).
    pub is_root: Option<bool>,

    /// Filter by tasks/ directory membership.
    pub in_tasks_dir: Option<bool>,

    /// Filter by docs/ directory membership.
    pub in_docs_dir: Option<bool>,

    /// Column to sort results by. Default: [`SortColumn::UpdatedAt`].
    pub sort_by: SortColumn,

    /// Sort order for results. Default: [`SortOrder::Descending`].
    pub sort_order: SortOrder,

    /// Maximum number of results to return.
    pub limit: Option<u32>,

    /// Number of results to skip (for pagination).
    pub offset: Option<u32>,
}

/// Document state determined by filesystem location and dependencies.
///
/// State is derived from two factors:
/// 1. Whether the document resides in a `.closed/` directory
/// 2. Whether the document has open (non-closed) blocked-by dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// Document is not closed and has no open blockers.
    /// Available for work.
    Open,

    /// Document has at least one open blocked-by dependency.
    /// Cannot be worked on until blockers are resolved.
    Blocked,

    /// Document resides in a `.closed/` subdirectory.
    /// Task has been completed.
    Closed,
}

/// Column to sort query results by.
///
/// Default is [`SortColumn::UpdatedAt`], which surfaces recently modified
/// documents first when combined with descending order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortColumn {
    /// Sort by last modification timestamp (default).
    #[default]
    UpdatedAt,

    /// Sort by document creation timestamp.
    CreatedAt,

    /// Sort by task closure timestamp.
    /// Documents without `closed_at` sort last.
    ClosedAt,

    /// Sort by task priority (P0-P4).
    /// Lower numbers indicate higher priority.
    Priority,

    /// Sort alphabetically by document name.
    Name,

    /// Sort alphabetically by file path.
    Path,

    /// Sort by local view count (tracked in SQLite index).
    /// Used by `lat overview` to surface frequently-referenced documents.
    ViewCount,
}

/// Sort order for query results.
///
/// Default is [`SortOrder::Descending`], which shows highest values or
/// most recent timestamps first.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort from highest to lowest (default).
    /// For timestamps: newest first. For priority: P0 first.
    #[default]
    Descending,

    /// Sort from lowest to highest.
    /// For timestamps: oldest first. For priority: P4 first.
    Ascending,
}

/// Builds a SELECT query for documents with the given filter.
pub fn build_filter_query(filter: &DocumentFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("SELECT * FROM documents WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    append_filter_conditions(&mut sql, &mut params, filter);
    append_sort_clause(&mut sql, filter);
    append_limit_offset(&mut sql, &mut params, filter);

    (sql, params)
}

/// Builds a COUNT query for documents with the given filter.
pub fn build_count_query(filter: &DocumentFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from("SELECT COUNT(*) FROM documents WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    append_filter_conditions(&mut sql, &mut params, filter);

    (sql, params)
}

impl DocumentFilter {
    /// Creates a new filter with default settings (excludes closed documents).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a filter that includes closed documents.
    pub fn including_closed() -> Self {
        Self { include_closed: true, ..Self::default() }
    }

    /// Sets the state filter.
    pub fn with_state(mut self, state: DocumentState) -> Self {
        self.state = Some(state);
        self
    }

    /// Sets the priority filter.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets a priority range filter.
    pub fn with_priority_range(mut self, min: u8, max: u8) -> Self {
        self.priority_range = Some((min, max));
        self
    }

    /// Sets the task type filter.
    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
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

    /// Filters by path prefix.
    pub fn with_path_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(prefix.into());
        self
    }

    /// Filters by name substring match.
    pub fn with_name_contains(mut self, substring: impl Into<String>) -> Self {
        self.name_contains = Some(substring.into());
        self
    }

    /// Filters by root document status.
    pub fn with_is_root(mut self, is_root: bool) -> Self {
        self.is_root = Some(is_root);
        self
    }

    /// Filters by tasks directory membership.
    pub fn with_in_tasks_dir(mut self, in_tasks_dir: bool) -> Self {
        self.in_tasks_dir = Some(in_tasks_dir);
        self
    }

    /// Filters by docs directory membership.
    pub fn with_in_docs_dir(mut self, in_docs_dir: bool) -> Self {
        self.in_docs_dir = Some(in_docs_dir);
        self
    }

    /// Sets the sort column.
    pub fn sort_by(mut self, column: SortColumn) -> Self {
        self.sort_by = column;
        self
    }

    /// Sets the sort order.
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    /// Limits the number of results.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the result offset for pagination.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filters to documents created at or after the given timestamp.
    pub fn with_created_after(mut self, dt: DateTime<Utc>) -> Self {
        self.created_after = Some(dt);
        self
    }

    /// Filters to documents created at or before the given timestamp.
    pub fn with_created_before(mut self, dt: DateTime<Utc>) -> Self {
        self.created_before = Some(dt);
        self
    }

    /// Filters to documents updated at or after the given timestamp.
    pub fn with_updated_after(mut self, dt: DateTime<Utc>) -> Self {
        self.updated_after = Some(dt);
        self
    }

    /// Filters to documents updated at or before the given timestamp.
    pub fn with_updated_before(mut self, dt: DateTime<Utc>) -> Self {
        self.updated_before = Some(dt);
        self
    }

    /// Filters to documents closed at or after the given timestamp.
    pub fn with_closed_after(mut self, dt: DateTime<Utc>) -> Self {
        self.closed_after = Some(dt);
        self
    }

    /// Filters to documents closed at or before the given timestamp.
    pub fn with_closed_before(mut self, dt: DateTime<Utc>) -> Self {
        self.closed_before = Some(dt);
        self
    }

    /// Filters to tasks discovered from the specified parent document.
    pub fn with_discovered_from(mut self, parent_id: impl Into<String>) -> Self {
        self.discovered_from = Some(parent_id.into());
        self
    }
}

fn append_filter_conditions(
    sql: &mut String,
    params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    filter: &DocumentFilter,
) {
    if !filter.include_closed {
        sql.push_str(" AND is_closed = 0");
    }

    if let Some(state) = filter.state {
        match state {
            DocumentState::Open => sql.push_str(" AND is_closed = 0"),
            DocumentState::Closed => sql.push_str(" AND is_closed = 1"),
            DocumentState::Blocked => {
                // NOTE: This query depends on links being stored with link_type = 'blocked_by'.
                // The link_queries module (dr-epv.4.4) is responsible for storing blocked-by
                // relationships with this link_type. Until that module is implemented, this
                // filter will return no results.
                sql.push_str(
                    " AND is_closed = 0 AND EXISTS (
                        SELECT 1 FROM links l
                        JOIN documents d2 ON l.target_id = d2.id
                        WHERE l.source_id = documents.id
                        AND l.link_type = 'blocked_by'
                        AND d2.is_closed = 0
                    )",
                );
            }
        }
    }

    if let Some(p) = filter.priority {
        sql.push_str(" AND priority = ?");
        params.push(Box::new(p as i32));
    }

    if let Some((min, max)) = filter.priority_range {
        sql.push_str(" AND priority >= ? AND priority <= ?");
        params.push(Box::new(min as i32));
        params.push(Box::new(max as i32));
    }

    if let Some(tt) = filter.task_type {
        sql.push_str(" AND task_type = ?");
        params.push(Box::new(tt.to_string()));
    }

    if !filter.labels_all.is_empty() {
        for label in &filter.labels_all {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM labels WHERE labels.document_id = documents.id AND labels.label = ?)",
            );
            params.push(Box::new(label.clone()));
        }
    }

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

    if let Some(prefix) = &filter.path_prefix {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    if let Some(substring) = &filter.name_contains {
        sql.push_str(" AND name LIKE ?");
        params.push(Box::new(format!("%{substring}%")));
    }

    if let Some(dt) = filter.created_after {
        sql.push_str(" AND created_at >= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(dt) = filter.created_before {
        sql.push_str(" AND created_at <= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(dt) = filter.updated_after {
        sql.push_str(" AND updated_at >= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(dt) = filter.updated_before {
        sql.push_str(" AND updated_at <= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(dt) = filter.closed_after {
        sql.push_str(" AND closed_at >= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(dt) = filter.closed_before {
        sql.push_str(" AND closed_at <= ?");
        params.push(Box::new(dt.to_rfc3339()));
    }

    if let Some(parent_id) = &filter.discovered_from {
        sql.push_str(
            " AND EXISTS (
                SELECT 1 FROM links l
                JOIN documents d2 ON l.target_id = d2.id
                WHERE l.source_id = documents.id
                AND l.link_type = 'discovered_from'
                AND d2.lattice_id = ?
            )",
        );
        params.push(Box::new(parent_id.clone()));
    }

    if let Some(v) = filter.is_root {
        sql.push_str(" AND is_root = ?");
        params.push(Box::new(v as i32));
    }

    if let Some(v) = filter.in_tasks_dir {
        sql.push_str(" AND in_tasks_dir = ?");
        params.push(Box::new(v as i32));
    }

    if let Some(v) = filter.in_docs_dir {
        sql.push_str(" AND in_docs_dir = ?");
        params.push(Box::new(v as i32));
    }
}

fn append_sort_clause(sql: &mut String, filter: &DocumentFilter) {
    let column = match filter.sort_by {
        SortColumn::UpdatedAt => "updated_at",
        SortColumn::CreatedAt => "created_at",
        SortColumn::ClosedAt => "closed_at",
        SortColumn::Priority => "priority",
        SortColumn::Name => "name",
        SortColumn::Path => "path",
        SortColumn::ViewCount => "view_count",
    };

    let order = match filter.sort_order {
        SortOrder::Ascending => "ASC",
        SortOrder::Descending => "DESC",
    };

    sql.push_str(&format!(" ORDER BY {column} {order}"));
}

fn append_limit_offset(
    sql: &mut String,
    params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    filter: &DocumentFilter,
) {
    if let Some(limit) = filter.limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }

    if let Some(offset) = filter.offset {
        if filter.limit.is_none() {
            sql.push_str(" LIMIT -1");
        }
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset as i64));
    }
}
