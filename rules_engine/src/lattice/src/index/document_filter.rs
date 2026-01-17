use chrono::{DateTime, Utc};

use crate::document::frontmatter_schema::TaskType;

/// Filter criteria for querying documents.
#[derive(Debug, Clone, Default)]
pub struct DocumentFilter {
    pub include_closed: bool,
    pub state: Option<DocumentState>,
    pub priority: Option<u8>,
    pub priority_range: Option<(u8, u8)>,
    pub task_type: Option<TaskType>,
    pub labels_all: Vec<String>,
    pub labels_any: Vec<String>,
    pub path_prefix: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub closed_after: Option<DateTime<Utc>>,
    pub closed_before: Option<DateTime<Utc>>,
    pub is_root: Option<bool>,
    pub in_tasks_dir: Option<bool>,
    pub in_docs_dir: Option<bool>,
    pub sort_by: SortColumn,
    pub sort_order: SortOrder,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Document state based on is_closed flag and blocked-by relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    Open,
    Blocked,
    Closed,
}

/// Column to sort results by.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortColumn {
    #[default]
    UpdatedAt,
    CreatedAt,
    ClosedAt,
    Priority,
    Name,
    Path,
    ViewCount,
}

/// Sort order for query results.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    #[default]
    Descending,
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
