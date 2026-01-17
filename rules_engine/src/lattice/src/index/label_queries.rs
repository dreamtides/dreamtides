use rusqlite::{Connection, params};
use tracing::{debug, info};

use crate::error::error_types::LatticeError;

/// A label with its usage count across documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelCount {
    pub label: String,
    pub count: u64,
}

/// Adds a label to a document.
///
/// If the label already exists on the document, this is a no-op.
pub fn add(conn: &Connection, document_id: &str, label: &str) -> Result<bool, LatticeError> {
    debug!(document_id, label, "Adding label to document");

    let result =
        conn.execute("INSERT OR IGNORE INTO labels (document_id, label) VALUES (?1, ?2)", params![
            document_id,
            label
        ]);

    match result {
        Ok(rows) if rows > 0 => {
            info!(document_id, label, "Label added to document");
            Ok(true)
        }
        Ok(_) => {
            debug!(document_id, label, "Label already exists on document");
            Ok(false)
        }
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to add label '{label}' to document {document_id}: {e}"),
        }),
    }
}

/// Removes a label from a document.
///
/// Returns true if the label was removed, false if it didn't exist.
pub fn remove(conn: &Connection, document_id: &str, label: &str) -> Result<bool, LatticeError> {
    debug!(document_id, label, "Removing label from document");

    let rows_affected = conn
        .execute("DELETE FROM labels WHERE document_id = ? AND label = ?", params![
            document_id,
            label
        ])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to remove label '{label}' from document {document_id}: {e}"),
        })?;

    if rows_affected > 0 {
        info!(document_id, label, "Label removed from document");
        Ok(true)
    } else {
        debug!(document_id, label, "Label not found on document");
        Ok(false)
    }
}

/// Checks if a document has a specific label.
pub fn has_label(conn: &Connection, document_id: &str, label: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM labels WHERE document_id = ? AND label = ?",
            params![document_id, label],
            |row| row.get(0),
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check label '{label}' on document {document_id}: {e}"),
        })?;

    Ok(count > 0)
}

/// Returns all labels for a document, sorted alphabetically.
pub fn get_labels(conn: &Connection, document_id: &str) -> Result<Vec<String>, LatticeError> {
    debug!(document_id, "Getting labels for document");

    let mut stmt = conn
        .prepare("SELECT label FROM labels WHERE document_id = ? ORDER BY label")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare labels query: {e}"),
        })?;

    let labels = stmt
        .query_map([document_id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query labels for {document_id}: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect labels: {e}"),
        })?;

    debug!(document_id, count = labels.len(), "Labels found");
    Ok(labels)
}

/// Finds all document IDs with a specific label.
pub fn find_by_label(conn: &Connection, label: &str) -> Result<Vec<String>, LatticeError> {
    debug!(label, "Finding documents by label");

    let mut stmt = conn
        .prepare("SELECT document_id FROM labels WHERE label = ? ORDER BY document_id")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare label search query: {e}"),
        })?;

    let document_ids = stmt
        .query_map([label], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query documents with label '{label}': {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect document IDs: {e}"),
        })?;

    debug!(label, count = document_ids.len(), "Documents found with label");
    Ok(document_ids)
}

/// Finds all document IDs that have ALL specified labels (AND semantics).
///
/// Returns only documents that have every label in the list.
pub fn find_by_labels_all(conn: &Connection, labels: &[&str]) -> Result<Vec<String>, LatticeError> {
    if labels.is_empty() {
        return Ok(Vec::new());
    }

    debug!(labels = ?labels, "Finding documents with all labels");

    // Build a query that counts matches and requires all labels to match.
    let placeholders: Vec<&str> = labels.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT document_id FROM labels WHERE label IN ({})
         GROUP BY document_id HAVING COUNT(DISTINCT label) = ?
         ORDER BY document_id",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare AND label query: {e}"),
    })?;

    // Bind all labels, then the required count.
    let label_count = labels.len() as i64;
    let mut params_vec: Vec<&dyn rusqlite::ToSql> =
        labels.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    params_vec.push(&label_count);

    let document_ids = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query documents with all labels: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect document IDs: {e}"),
        })?;

    debug!(labels = ?labels, count = document_ids.len(), "Documents found with all labels");
    Ok(document_ids)
}

/// Finds all document IDs that have ANY of the specified labels (OR semantics).
///
/// Returns documents that have at least one label in the list.
pub fn find_by_labels_any(conn: &Connection, labels: &[&str]) -> Result<Vec<String>, LatticeError> {
    if labels.is_empty() {
        return Ok(Vec::new());
    }

    debug!(labels = ?labels, "Finding documents with any labels");

    let placeholders: Vec<&str> = labels.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT DISTINCT document_id FROM labels WHERE label IN ({}) ORDER BY document_id",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare OR label query: {e}"),
    })?;

    let params_vec: Vec<&dyn rusqlite::ToSql> =
        labels.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

    let document_ids = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query documents with any labels: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect document IDs: {e}"),
        })?;

    debug!(labels = ?labels, count = document_ids.len(), "Documents found with any labels");
    Ok(document_ids)
}

/// Lists all unique labels in the repository with their document counts.
///
/// Returns labels sorted alphabetically with the number of documents having
/// each label.
pub fn list_all(conn: &Connection) -> Result<Vec<LabelCount>, LatticeError> {
    debug!("Listing all labels with counts");

    let mut stmt = conn
        .prepare("SELECT label, COUNT(*) as count FROM labels GROUP BY label ORDER BY label")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare list all labels query: {e}"),
        })?;

    let labels = stmt
        .query_map([], |row| {
            Ok(LabelCount { label: row.get(0)?, count: row.get::<_, i64>(1)? as u64 })
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query all labels: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect labels: {e}"),
        })?;

    debug!(count = labels.len(), "Labels found");
    Ok(labels)
}

/// Adds a label to multiple documents in a single transaction.
///
/// Returns the number of documents that actually received the new label
/// (excludes documents that already had it).
pub fn add_to_multiple(
    conn: &Connection,
    document_ids: &[&str],
    label: &str,
) -> Result<usize, LatticeError> {
    if document_ids.is_empty() {
        return Ok(0);
    }

    debug!(label, count = document_ids.len(), "Adding label to multiple documents");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    let mut added_count = 0;
    for document_id in document_ids {
        let rows = tx
            .execute("INSERT OR IGNORE INTO labels (document_id, label) VALUES (?1, ?2)", params![
                document_id,
                label
            ])
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!("Failed to add label '{label}' to document {document_id}: {e}"),
            })?;
        added_count += rows;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit batch label addition: {e}"),
    })?;

    info!(label, added = added_count, total = document_ids.len(), "Label added to documents");
    Ok(added_count)
}

/// Removes a label from multiple documents in a single transaction.
///
/// Returns the number of documents that actually had the label removed.
pub fn remove_from_multiple(
    conn: &Connection,
    document_ids: &[&str],
    label: &str,
) -> Result<usize, LatticeError> {
    if document_ids.is_empty() {
        return Ok(0);
    }

    debug!(label, count = document_ids.len(), "Removing label from multiple documents");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    let mut removed_count = 0;
    for document_id in document_ids {
        let rows = tx
            .execute("DELETE FROM labels WHERE document_id = ? AND label = ?", params![
                document_id,
                label
            ])
            .map_err(|e| LatticeError::DatabaseError {
                reason: format!(
                    "Failed to remove label '{label}' from document {document_id}: {e}"
                ),
            })?;
        removed_count += rows;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit batch label removal: {e}"),
    })?;

    info!(
        label,
        removed = removed_count,
        total = document_ids.len(),
        "Label removed from documents"
    );
    Ok(removed_count)
}

/// Replaces all labels for a document with a new set.
///
/// Deletes existing labels and inserts the new ones in a single transaction.
pub fn sync_labels(
    conn: &Connection,
    document_id: &str,
    labels: &[&str],
) -> Result<(), LatticeError> {
    debug!(document_id, labels = ?labels, "Syncing labels for document");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    // Delete all existing labels.
    tx.execute("DELETE FROM labels WHERE document_id = ?", [document_id]).map_err(|e| {
        LatticeError::DatabaseError {
            reason: format!("Failed to clear labels for document {document_id}: {e}"),
        }
    })?;

    // Insert new labels.
    for label in labels {
        tx.execute("INSERT INTO labels (document_id, label) VALUES (?1, ?2)", params![
            document_id,
            label
        ])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to add label '{label}' to document {document_id}: {e}"),
        })?;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit label sync: {e}"),
    })?;

    info!(document_id, count = labels.len(), "Labels synced for document");
    Ok(())
}

/// Deletes all labels for a document.
///
/// Returns the number of labels deleted.
pub fn delete_for_document(conn: &Connection, document_id: &str) -> Result<usize, LatticeError> {
    debug!(document_id, "Deleting all labels for document");

    let rows_affected = conn
        .execute("DELETE FROM labels WHERE document_id = ?", [document_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete labels for document {document_id}: {e}"),
        })?;

    if rows_affected > 0 {
        info!(document_id, deleted = rows_affected, "Labels deleted for document");
    } else {
        debug!(document_id, "No labels to delete for document");
    }

    Ok(rows_affected)
}

/// Counts the number of labels on a document.
pub fn count_labels(conn: &Connection, document_id: &str) -> Result<u64, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM labels WHERE document_id = ?", [document_id], |row| {
            row.get(0)
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to count labels for document {document_id}: {e}"),
        })?;

    Ok(count as u64)
}

/// Counts the number of documents with a specific label.
pub fn count_documents_with_label(conn: &Connection, label: &str) -> Result<u64, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM labels WHERE label = ?", [label], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to count documents with label '{label}': {e}"),
        })?;

    Ok(count as u64)
}

/// Checks if a label exists anywhere in the repository.
pub fn label_exists(conn: &Connection, label: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM labels WHERE label = ?", [label], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check if label '{label}' exists: {e}"),
        })?;

    Ok(count > 0)
}

/// Returns the total count of label assignments in the repository.
pub fn total_count(conn: &Connection) -> Result<u64, LatticeError> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM labels", [], |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError { reason: format!("Failed to count total labels: {e}") }
        })?;

    Ok(count as u64)
}
