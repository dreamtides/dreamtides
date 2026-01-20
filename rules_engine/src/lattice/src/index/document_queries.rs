use rusqlite::{Connection, OptionalExtension, params};
use tracing::{debug, info};

use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_types::{DocumentRow, InsertDocument, UpdateBuilder};
use crate::index::{document_filter, document_types};

/// Inserts a single document into the index.
pub fn insert(conn: &Connection, doc: &InsertDocument) -> Result<(), LatticeError> {
    debug!(id = doc.id, path = doc.path, "Inserting document into index");

    conn.execute(
        "INSERT INTO documents (
            id, parent_id, path, name, description, task_type, is_closed, priority,
            created_at, updated_at, closed_at, body_hash, indexed_at, content_length,
            is_root, in_tasks_dir, in_docs_dir, skill
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'), ?13, ?14, ?15, ?16, ?17)",
        params![
            doc.id,
            doc.parent_id,
            doc.path,
            doc.name,
            doc.description,
            doc.task_type.map(|t| t.to_string()),
            doc.is_closed as i32,
            doc.priority,
            doc.created_at.map(|dt| dt.to_rfc3339()),
            doc.updated_at.map(|dt| dt.to_rfc3339()),
            doc.closed_at.map(|dt| dt.to_rfc3339()),
            doc.body_hash,
            doc.content_length,
            doc.is_root as i32,
            doc.in_tasks_dir as i32,
            doc.in_docs_dir as i32,
            doc.skill as i32,
        ],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to insert document {}: {e}", doc.id),
    })?;

    info!(id = doc.id, "Document inserted into index");
    Ok(())
}

/// Inserts multiple documents in a single transaction.
pub fn insert_batch(conn: &Connection, docs: &[InsertDocument]) -> Result<(), LatticeError> {
    if docs.is_empty() {
        return Ok(());
    }

    debug!(count = docs.len(), "Inserting batch of documents");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    for doc in docs {
        insert(&tx, doc)?;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit batch insert: {e}"),
    })?;

    info!(count = docs.len(), "Batch insert complete");
    Ok(())
}

/// Updates document metadata by ID using the provided update builder.
pub fn update(conn: &Connection, id: &str, builder: &UpdateBuilder) -> Result<bool, LatticeError> {
    debug!(id, "Updating document in index");

    let mut sql = String::from("UPDATE documents SET indexed_at = datetime('now')");
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(v) = builder.get_parent_id() {
        sql.push_str(", parent_id = ?");
        params_vec.push(Box::new(v.map(str::to_string)));
    }
    if let Some(v) = builder.get_path() {
        sql.push_str(", path = ?");
        params_vec.push(Box::new(v.to_string()));
    }
    if let Some(v) = builder.get_name() {
        sql.push_str(", name = ?");
        params_vec.push(Box::new(v.to_string()));
    }
    if let Some(v) = builder.get_description() {
        sql.push_str(", description = ?");
        params_vec.push(Box::new(v.to_string()));
    }
    if let Some(v) = builder.get_task_type() {
        sql.push_str(", task_type = ?");
        params_vec.push(Box::new(v.map(|t| t.to_string())));
    }
    if let Some(v) = builder.get_is_closed() {
        sql.push_str(", is_closed = ?");
        params_vec.push(Box::new(v as i32));
    }
    if let Some(v) = builder.get_priority() {
        sql.push_str(", priority = ?");
        params_vec.push(Box::new(v.map(|p| p as i32)));
    }
    if let Some(v) = builder.get_updated_at() {
        sql.push_str(", updated_at = ?");
        params_vec.push(Box::new(v.to_rfc3339()));
    }
    if let Some(v) = builder.get_closed_at() {
        sql.push_str(", closed_at = ?");
        params_vec.push(Box::new(v.map(|dt| dt.to_rfc3339())));
    }
    if let Some(v) = builder.get_body_hash() {
        sql.push_str(", body_hash = ?");
        params_vec.push(Box::new(v.to_string()));
    }
    if let Some(v) = builder.get_content_length() {
        sql.push_str(", content_length = ?");
        params_vec.push(Box::new(v));
    }
    if let Some(v) = builder.get_is_root() {
        sql.push_str(", is_root = ?");
        params_vec.push(Box::new(v as i32));
    }
    if let Some(v) = builder.get_in_tasks_dir() {
        sql.push_str(", in_tasks_dir = ?");
        params_vec.push(Box::new(v as i32));
    }
    if let Some(v) = builder.get_in_docs_dir() {
        sql.push_str(", in_docs_dir = ?");
        params_vec.push(Box::new(v as i32));
    }
    if let Some(v) = builder.get_skill() {
        sql.push_str(", skill = ?");
        params_vec.push(Box::new(v as i32));
    }

    sql.push_str(" WHERE id = ?");
    params_vec.push(Box::new(id.to_string()));

    let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(AsRef::as_ref).collect();
    let rows_affected = conn.execute(&sql, params.as_slice()).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to update document {id}: {e}") }
    })?;

    if rows_affected > 0 {
        debug!(id, "Document updated");
    } else {
        debug!(id, "Document not found for update");
    }

    Ok(rows_affected > 0)
}

/// Updates multiple documents atomically with the same changes.
pub fn update_batch(
    conn: &Connection,
    ids: &[&str],
    builder: &UpdateBuilder,
) -> Result<usize, LatticeError> {
    if ids.is_empty() {
        return Ok(0);
    }

    debug!(count = ids.len(), "Batch updating documents");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    let mut total_updated = 0;
    for &id in ids {
        if update(&tx, id, builder)? {
            total_updated += 1;
        }
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit batch update: {e}"),
    })?;

    info!(updated = total_updated, "Batch update complete");
    Ok(total_updated)
}

/// Deletes a document by ID.
pub fn delete_by_id(conn: &Connection, id: &str) -> Result<bool, LatticeError> {
    debug!(id, "Deleting document from index");

    let rows_affected = conn.execute("DELETE FROM documents WHERE id = ?", [id]).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to delete document {id}: {e}") }
    })?;

    if rows_affected > 0 {
        info!(id, "Document deleted from index");
    } else {
        debug!(id, "Document not found for deletion");
    }

    Ok(rows_affected > 0)
}

/// Deletes multiple documents by ID.
pub fn delete_batch(conn: &Connection, ids: &[&str]) -> Result<usize, LatticeError> {
    if ids.is_empty() {
        return Ok(0);
    }

    debug!(count = ids.len(), "Batch deleting documents");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    let mut total_deleted = 0;
    for &id in ids {
        if delete_by_id(&tx, id)? {
            total_deleted += 1;
        }
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit batch delete: {e}"),
    })?;

    info!(deleted = total_deleted, "Batch delete complete");
    Ok(total_deleted)
}

/// Deletes all documents whose path starts with the given prefix.
pub fn delete_by_path_prefix(conn: &Connection, prefix: &str) -> Result<usize, LatticeError> {
    debug!(prefix, "Deleting documents by path prefix");

    let pattern = format!("{prefix}%");
    let rows_affected = conn
        .execute("DELETE FROM documents WHERE path LIKE ?", [&pattern])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete documents with prefix {prefix}: {e}"),
        })?;

    info!(deleted = rows_affected, prefix, "Documents deleted by path prefix");
    Ok(rows_affected)
}

/// Looks up a document by its Lattice ID.
pub fn lookup_by_id(conn: &Connection, id: &str) -> Result<Option<DocumentRow>, LatticeError> {
    debug!(id, "Looking up document by ID");

    let result = conn
        .query_row("SELECT * FROM documents WHERE id = ?", [id], document_types::row_to_document)
        .optional()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to lookup document {id}: {e}"),
        })?;

    if result.is_some() {
        debug!(id, "Document found");
    } else {
        debug!(id, "Document not found");
    }

    Ok(result)
}

/// Looks up a document by its file path.
pub fn lookup_by_path(conn: &Connection, path: &str) -> Result<Option<DocumentRow>, LatticeError> {
    debug!(path, "Looking up document by path");

    let result = conn
        .query_row(
            "SELECT * FROM documents WHERE path = ?",
            [path],
            document_types::row_to_document,
        )
        .optional()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to lookup document at path {path}: {e}"),
        })?;

    if result.is_some() {
        debug!(path, "Document found");
    } else {
        debug!(path, "Document not found");
    }

    Ok(result)
}

/// Looks up documents by name (may return multiple if names are not unique).
pub fn lookup_by_name(conn: &Connection, name: &str) -> Result<Vec<DocumentRow>, LatticeError> {
    debug!(name, "Looking up documents by name");

    let mut stmt = conn.prepare("SELECT * FROM documents WHERE name = ?").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to prepare name lookup: {e}") }
    })?;

    let rows = stmt
        .query_map([name], document_types::row_to_document)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query documents by name {name}: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect name lookup results: {e}"),
        })?;

    debug!(name, count = rows.len(), "Documents found by name");
    Ok(rows)
}

/// Queries documents with the given filter criteria.
pub fn query(conn: &Connection, filter: &DocumentFilter) -> Result<Vec<DocumentRow>, LatticeError> {
    let (sql, params) = document_filter::build_filter_query(filter);
    debug!(sql = sql.as_str(), "Executing document query");

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare filter query: {e}"),
    })?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();
    let rows = stmt
        .query_map(params_refs.as_slice(), document_types::row_to_document)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to execute filter query: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect filter query results: {e}"),
        })?;

    debug!(count = rows.len(), "Filter query returned results");
    Ok(rows)
}

/// Counts documents matching the given filter.
pub fn count(conn: &Connection, filter: &DocumentFilter) -> Result<u64, LatticeError> {
    let (sql, params) = document_filter::build_count_query(filter);
    debug!(sql = sql.as_str(), "Executing document count query");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();
    let count: i64 =
        conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError { reason: format!("Failed to count documents: {e}") }
        })?;

    debug!(count, "Document count query result");
    Ok(count as u64)
}

/// Checks if a document with the given ID exists.
pub fn exists(conn: &Connection, id: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents WHERE id = ?", [id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check document existence: {e}"),
        })?;

    Ok(count > 0)
}

/// Checks if a document exists at the given path.
pub fn exists_at_path(conn: &Connection, path: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents WHERE path = ?", [path], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check path existence: {e}"),
        })?;

    Ok(count > 0)
}

/// Returns all document IDs in the index.
pub fn all_ids(conn: &Connection) -> Result<Vec<String>, LatticeError> {
    let mut stmt = conn.prepare("SELECT id FROM documents").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to prepare all_ids query: {e}") }
    })?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query all document IDs: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect document IDs: {e}"),
        })?;

    Ok(rows)
}

/// Returns document IDs matching the given prefix, limited to a maximum count.
///
/// Used for shell completion to provide dynamic ID suggestions. Returns IDs
/// sorted alphabetically for consistent ordering.
pub fn ids_by_prefix(
    conn: &Connection,
    prefix: Option<&str>,
    limit: usize,
) -> Result<Vec<String>, LatticeError> {
    debug!(prefix = prefix, limit, "Querying IDs by prefix");

    let (sql, params): (&str, Vec<&dyn rusqlite::ToSql>) = match prefix {
        Some(p) if !p.is_empty() => {
            let pattern = format!("{p}%");
            // Need to keep pattern alive for the borrow
            let sql = "SELECT id FROM documents WHERE id LIKE ?1 ORDER BY id LIMIT ?2";
            (sql, vec![])
        }
        _ => ("SELECT id FROM documents ORDER BY id LIMIT ?1", vec![]),
    };

    let rows = if let Some(p) = prefix.filter(|s| !s.is_empty()) {
        let pattern = format!("{p}%");
        let mut stmt = conn.prepare(sql).map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare ids_by_prefix query: {e}"),
        })?;
        stmt.query_map(params![pattern, limit as i64], |row| row.get(0))
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to query IDs by prefix: {e}"),
            })?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to collect IDs: {e}"),
            })?
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare ids_by_prefix query: {e}"),
        })?;
        stmt.query_map(params![limit as i64], |row| row.get(0))
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to query all IDs: {e}"),
            })?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to collect IDs: {e}"),
            })?
    };

    debug!(count = rows.len(), "IDs returned for completion");
    Ok(rows)
}

/// Returns all document paths in the index.
pub fn all_paths(conn: &Connection) -> Result<Vec<String>, LatticeError> {
    let mut stmt = conn.prepare("SELECT path FROM documents").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to prepare all_paths query: {e}") }
    })?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query all document paths: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect document paths: {e}"),
        })?;

    Ok(rows)
}

/// Returns document IDs matching the given prefix, limited to a maximum count.
///
/// Used for shell completion to provide dynamic ID suggestions. Returns IDs
/// sorted alphabetically for consistent ordering.
pub fn ids_by_prefix(
    conn: &Connection,
    prefix: Option<&str>,
    limit: usize,
) -> Result<Vec<String>, LatticeError> {
    debug!(prefix = prefix, limit, "Querying IDs by prefix");

    let rows = if let Some(p) = prefix.filter(|s| !s.is_empty()) {
        let pattern = format!("{p}%");
        let mut stmt = conn
            .prepare("SELECT id FROM documents WHERE id LIKE ?1 ORDER BY id LIMIT ?2")
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to prepare ids_by_prefix query: {e}"),
            })?;
        stmt.query_map(params![pattern, limit as i64], |row| row.get(0))
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to query IDs by prefix: {e}"),
            })?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to collect IDs: {e}"),
            })?
    } else {
        let mut stmt =
            conn.prepare("SELECT id FROM documents ORDER BY id LIMIT ?1").map_err(|e| {
                LatticeError::DatabaseError {
                    reason: format!("Failed to prepare ids_by_prefix query: {e}"),
                }
            })?;
        stmt.query_map(params![limit as i64], |row| row.get(0))
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to query all IDs: {e}"),
            })?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to collect IDs: {e}"),
            })?
    };

    debug!(count = rows.len(), "IDs returned for completion");
    Ok(rows)
}
