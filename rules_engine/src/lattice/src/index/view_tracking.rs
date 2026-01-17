use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use tracing::{debug, info};

use crate::error::error_types::LatticeError;

/// View data for a single document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewData {
    pub document_id: String,
    pub view_count: u64,
    pub last_viewed: DateTime<Utc>,
}

/// Aggregate statistics for all views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewStats {
    pub tracked_documents: u64,
    pub total_views: u64,
}

/// Records a view for a document.
///
/// Increments the view count and updates the last_viewed timestamp. If no
/// view record exists for the document, creates one with view_count = 1.
///
/// The documents.view_count column is automatically updated via database
/// trigger to maintain the denormalized copy.
pub fn record_view(conn: &Connection, document_id: &str) -> Result<u64, LatticeError> {
    let timestamp = Utc::now().to_rfc3339();
    debug!(document_id, timestamp = %timestamp, "Recording document view");

    // Use INSERT OR REPLACE to atomically insert or update the view record.
    // If the document already has a view record, increment the count.
    // If not, create a new record with count = 1.
    let rows_affected = conn
        .execute(
            "INSERT INTO views (document_id, view_count, last_viewed)
             VALUES (?1, 1, ?2)
             ON CONFLICT(document_id) DO UPDATE SET
                view_count = view_count + 1,
                last_viewed = excluded.last_viewed",
            params![document_id, timestamp],
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to record view for document {document_id}: {e}"),
        })?;

    if rows_affected > 0 {
        // Query the new count to return it
        let new_count = get_view_count(conn, document_id)?;
        info!(document_id, view_count = new_count, "Document view recorded");
        Ok(new_count)
    } else {
        // This shouldn't happen with INSERT OR REPLACE, but handle gracefully
        debug!(document_id, "No rows affected when recording view");
        Ok(0)
    }
}

/// Returns the view count for a document.
///
/// Returns 0 if the document has no view records.
pub fn get_view_count(conn: &Connection, document_id: &str) -> Result<u64, LatticeError> {
    debug!(document_id, "Getting view count for document");

    let result: Result<i64, _> = conn.query_row(
        "SELECT view_count FROM views WHERE document_id = ?",
        [document_id],
        |row| row.get(0),
    );

    match result {
        Ok(count) => Ok(count as u64),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to get view count for document {document_id}: {e}"),
        }),
    }
}

/// Returns the last viewed timestamp for a document.
///
/// Returns None if the document has no view records.
pub fn get_last_viewed(
    conn: &Connection,
    document_id: &str,
) -> Result<Option<DateTime<Utc>>, LatticeError> {
    debug!(document_id, "Getting last viewed timestamp for document");

    let result: Result<String, _> = conn.query_row(
        "SELECT last_viewed FROM views WHERE document_id = ?",
        [document_id],
        |row| row.get(0),
    );

    match result {
        Ok(timestamp_str) => {
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| LatticeError::DatabaseError {
                    reason: format!(
                        "Invalid timestamp format in views table for {document_id}: {e}"
                    ),
                })?;
            Ok(Some(timestamp))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to get last viewed for document {document_id}: {e}"),
        }),
    }
}

/// Returns full view data for a document.
///
/// Returns None if the document has no view records.
pub fn get_view_data(
    conn: &Connection,
    document_id: &str,
) -> Result<Option<ViewData>, LatticeError> {
    debug!(document_id, "Getting view data for document");

    let result = conn.query_row(
        "SELECT document_id, view_count, last_viewed FROM views WHERE document_id = ?",
        [document_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
    );

    match result {
        Ok((id, count, timestamp_str)) => {
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| LatticeError::DatabaseError {
                    reason: format!(
                        "Invalid timestamp format in views table for {document_id}: {e}"
                    ),
                })?;
            Ok(Some(ViewData { document_id: id, view_count: count as u64, last_viewed: timestamp }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to get view data for document {document_id}: {e}"),
        }),
    }
}

/// Returns aggregate view statistics.
pub fn get_view_stats(conn: &Connection) -> Result<ViewStats, LatticeError> {
    debug!("Getting view statistics");

    let (tracked_documents, total_views): (i64, i64) = conn
        .query_row("SELECT COUNT(*), COALESCE(SUM(view_count), 0) FROM views", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to get view statistics: {e}"),
        })?;

    Ok(ViewStats { tracked_documents: tracked_documents as u64, total_views: total_views as u64 })
}

/// Resets all view counts by clearing the views table.
///
/// This is used by `lat overview --reset-views` to clear view history.
/// The documents.view_count column is automatically reset to 0 via database
/// trigger.
pub fn reset_all_views(conn: &Connection) -> Result<u64, LatticeError> {
    debug!("Resetting all view counts");

    let rows_deleted = conn.execute("DELETE FROM views", []).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to reset view counts: {e}") }
    })?;

    info!(documents_cleared = rows_deleted, "View history cleared");
    Ok(rows_deleted as u64)
}

/// Deletes view data for a specific document.
///
/// This is typically called when a document is removed from the index.
/// Returns true if a view record was deleted, false if none existed.
pub fn delete_view(conn: &Connection, document_id: &str) -> Result<bool, LatticeError> {
    debug!(document_id, "Deleting view data for document");

    let rows_deleted = conn
        .execute("DELETE FROM views WHERE document_id = ?", [document_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete view data for document {document_id}: {e}"),
        })?;

    if rows_deleted > 0 {
        info!(document_id, "View data deleted for document");
        Ok(true)
    } else {
        debug!(document_id, "No view data to delete for document");
        Ok(false)
    }
}
