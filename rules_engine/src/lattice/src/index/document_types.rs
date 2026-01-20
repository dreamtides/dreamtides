//! Index-specific document types for SQLite query results.
//!
//! This module defines types for document metadata stored in the SQLite index,
//! distinct from the [`Document`] type used when parsing documents from the
//! filesystem.
//!
//! # Type Distinction
//!
//! **[`DocumentRow`]** represents cached metadata from the index:
//! - Contains all frontmatter fields (id, name, description, task_type, etc.)
//! - Includes computed fields derived from paths (is_closed, is_root)
//! - Includes index-specific fields (indexed_at, body_hash, link/backlink
//!   counts)
//! - Does NOT contain the document body (only content_length for size checks)
//!
//! **[`Document`]** (in `document::document_reader`) represents a fully parsed
//! document:
//! - Contains the parsed [`Frontmatter`] struct
//! - Contains the original YAML string (for round-trip fidelity)
//! - Contains the full markdown body content
//!
//! # When to Use Each Type
//!
//! Use **`DocumentRow`** for:
//! - Listing and filtering documents (`lat list`, `lat ready`)
//! - Querying metadata without reading files (`lat blocked`, `lat stale`)
//! - Computing statistics and aggregations (`lat stats`)
//! - Any operation where body content is not needed
//!
//! Use **`Document`** for:
//! - Displaying full document content (`lat show` without `--short`)
//! - Template composition (reading `[Lattice] Context` sections)
//! - Document modification and formatting (`lat fmt`, `lat update`)
//! - Any operation that reads or writes body content
//!
//! # Additional Types
//!
//! - [`InsertDocument`]: Data for inserting new documents into the index
//! - [`UpdateBuilder`]: Builder pattern for partial document updates
//!
//! [`Document`]: crate::document::document_reader::Document
//! [`Frontmatter`]: crate::document::frontmatter_schema::Frontmatter

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Error as SqliteError, Row};

use crate::document::frontmatter_schema::TaskType;

/// A document row from the SQLite index.
///
/// This type represents cached document metadata for efficient queries without
/// filesystem access. It contains all frontmatter fields plus index-specific
/// computed fields, but excludes the document body.
#[derive(Debug, Clone)]
pub struct DocumentRow {
    pub id: String,
    pub parent_id: Option<String>,
    pub path: String,
    pub name: String,
    pub description: String,
    pub task_type: Option<TaskType>,
    pub is_closed: bool,
    pub priority: Option<u8>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub body_hash: String,
    pub indexed_at: DateTime<Utc>,
    pub content_length: i64,
    pub link_count: i32,
    pub backlink_count: i32,
    pub view_count: i32,
    pub is_root: bool,
    pub skill: bool,
}

/// Data for inserting a new document into the index.
#[derive(Debug, Clone)]
pub struct InsertDocument {
    pub id: String,
    pub parent_id: Option<String>,
    pub path: String,
    pub name: String,
    pub description: String,
    pub task_type: Option<TaskType>,
    pub is_closed: bool,
    pub priority: Option<u8>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub body_hash: String,
    pub content_length: i64,
    pub is_root: bool,
    pub skill: bool,
}

/// Builder for updating document fields.
///
/// Fields use `Option<Option<T>>` to distinguish between:
/// - `None`: don't update this field
/// - `Some(None)`: explicitly set this field to NULL
/// - `Some(Some(value))`: set this field to value
#[derive(Debug, Clone, Default)]
#[expect(clippy::option_option)]
pub struct UpdateBuilder<'a> {
    parent_id: Option<Option<&'a str>>,
    path: Option<&'a str>,
    name: Option<&'a str>,
    description: Option<&'a str>,
    task_type: Option<Option<TaskType>>,
    is_closed: Option<bool>,
    priority: Option<Option<u8>>,
    updated_at: Option<DateTime<Utc>>,
    closed_at: Option<Option<DateTime<Utc>>>,
    body_hash: Option<&'a str>,
    content_length: Option<i64>,
    is_root: Option<bool>,
    skill: Option<bool>,
}

/// Converts a SQLite row to a DocumentRow.
pub fn row_to_document(row: &Row) -> Result<DocumentRow, SqliteError> {
    let task_type_str: Option<String> = row.get("task_type")?;
    let task_type = task_type_str.and_then(|s| s.parse::<TaskType>().ok());

    let created_at_str: Option<String> = row.get("created_at")?;
    let created_at = created_at_str
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let updated_at_str: Option<String> = row.get("updated_at")?;
    let updated_at = updated_at_str
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let closed_at_str: Option<String> = row.get("closed_at")?;
    let closed_at = closed_at_str
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let indexed_at_str: String = row.get("indexed_at")?;
    let indexed_at = DateTime::parse_from_rfc3339(&indexed_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let is_closed: i32 = row.get("is_closed")?;
    let is_root: i32 = row.get("is_root")?;
    let skill: i32 = row.get("skill")?;

    let priority_i32: Option<i32> = row.get("priority")?;
    let priority = priority_i32.map(|p| p as u8);

    Ok(DocumentRow {
        id: row.get("id")?,
        parent_id: row.get("parent_id")?,
        path: row.get("path")?,
        name: row.get("name")?,
        description: row.get("description")?,
        task_type,
        is_closed: is_closed != 0,
        priority,
        created_at,
        updated_at,
        closed_at,
        body_hash: row.get("body_hash")?,
        indexed_at,
        content_length: row.get("content_length")?,
        link_count: row.get("link_count")?,
        backlink_count: row.get("backlink_count")?,
        view_count: row.get("view_count")?,
        is_root: is_root != 0,
        skill: skill != 0,
    })
}

impl<'a> UpdateBuilder<'a> {
    /// Creates a new empty update builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the parent_id (Some(None) clears it, Some(Some(id)) sets it).
    pub fn parent_id(mut self, parent_id: Option<&'a str>) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets the path.
    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    /// Sets the name.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the task_type (Some(None) clears it).
    pub fn task_type(mut self, task_type: Option<TaskType>) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Sets is_closed.
    pub fn is_closed(mut self, is_closed: bool) -> Self {
        self.is_closed = Some(is_closed);
        self
    }

    /// Sets priority (Some(None) clears it).
    pub fn priority(mut self, priority: Option<u8>) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets the updated_at timestamp.
    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    /// Sets closed_at (Some(None) clears it).
    pub fn closed_at(mut self, closed_at: Option<DateTime<Utc>>) -> Self {
        self.closed_at = Some(closed_at);
        self
    }

    /// Sets the body hash.
    pub fn body_hash(mut self, body_hash: &'a str) -> Self {
        self.body_hash = Some(body_hash);
        self
    }

    /// Sets the content length.
    pub fn content_length(mut self, content_length: i64) -> Self {
        self.content_length = Some(content_length);
        self
    }

    /// Sets is_root.
    pub fn is_root(mut self, is_root: bool) -> Self {
        self.is_root = Some(is_root);
        self
    }

    /// Returns the parent_id update value if set.
    #[expect(clippy::option_option)]
    pub(crate) fn get_parent_id(&self) -> Option<Option<&'a str>> {
        self.parent_id
    }

    /// Returns the path update value if set.
    pub(crate) fn get_path(&self) -> Option<&'a str> {
        self.path
    }

    /// Returns the name update value if set.
    pub(crate) fn get_name(&self) -> Option<&'a str> {
        self.name
    }

    /// Returns the description update value if set.
    pub(crate) fn get_description(&self) -> Option<&'a str> {
        self.description
    }

    /// Returns the task_type update value if set.
    #[expect(clippy::option_option)]
    pub(crate) fn get_task_type(&self) -> Option<Option<TaskType>> {
        self.task_type
    }

    /// Returns the is_closed update value if set.
    pub(crate) fn get_is_closed(&self) -> Option<bool> {
        self.is_closed
    }

    /// Returns the priority update value if set.
    #[expect(clippy::option_option)]
    pub(crate) fn get_priority(&self) -> Option<Option<u8>> {
        self.priority
    }

    /// Returns the updated_at update value if set.
    pub(crate) fn get_updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }

    /// Returns the closed_at update value if set.
    #[expect(clippy::option_option)]
    pub(crate) fn get_closed_at(&self) -> Option<Option<DateTime<Utc>>> {
        self.closed_at
    }

    /// Returns the body_hash update value if set.
    pub(crate) fn get_body_hash(&self) -> Option<&'a str> {
        self.body_hash
    }

    /// Returns the content_length update value if set.
    pub(crate) fn get_content_length(&self) -> Option<i64> {
        self.content_length
    }

    /// Returns the is_root update value if set.
    pub(crate) fn get_is_root(&self) -> Option<bool> {
        self.is_root
    }

    /// Sets skill.
    pub fn skill(mut self, skill: bool) -> Self {
        self.skill = Some(skill);
        self
    }

    /// Returns the skill update value if set.
    pub(crate) fn get_skill(&self) -> Option<bool> {
        self.skill
    }
}

impl InsertDocument {
    #[expect(clippy::too_many_arguments)]
    /// Creates an InsertDocument from document fields with path analysis.
    pub fn new(
        id: String,
        parent_id: Option<String>,
        path: String,
        name: String,
        description: String,
        task_type: Option<TaskType>,
        priority: Option<u8>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        closed_at: Option<DateTime<Utc>>,
        body_hash: String,
        content_length: i64,
        skill: bool,
    ) -> Self {
        let is_closed = path.contains("/.closed/");
        let is_root = compute_is_root(&path);

        Self {
            id,
            parent_id,
            path,
            name,
            description,
            task_type,
            is_closed,
            priority,
            created_at,
            updated_at,
            closed_at,
            body_hash,
            content_length,
            is_root,
            skill,
        }
    }
}

fn compute_is_root(path: &str) -> bool {
    let path = Path::new(path);
    let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    file_stem == parent_name || file_stem.starts_with("00_")
}
