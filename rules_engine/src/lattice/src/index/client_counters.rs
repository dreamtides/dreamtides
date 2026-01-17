use rusqlite::{Connection, OptionalExtension};
use tracing::debug;

use crate::error::error_types::LatticeError;

/// Gets the next counter value for a client, creating the record if needed.
///
/// Returns the current counter value and increments it in the database.
/// This is an atomic operation suitable for ID generation - uses a single
/// SQL statement with UPSERT and RETURNING to avoid race conditions.
pub fn get_and_increment(conn: &Connection, client_id: &str) -> Result<u64, LatticeError> {
    debug!(client_id, "Getting and incrementing counter");

    // Atomic upsert: inserts with next_counter=1 (returning 0) or increments
    // existing (returning old value). Single statement prevents race conditions.
    let counter: i64 = conn
        .query_row(
            "INSERT INTO client_counters (client_id, next_counter) VALUES (?1, 1)
             ON CONFLICT(client_id) DO UPDATE SET next_counter = next_counter + 1
             RETURNING next_counter - 1",
            [client_id],
            |row| row.get(0),
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to get and increment counter for client {client_id}: {e}"),
        })?;

    debug!(client_id, counter, "Got and incremented counter");
    Ok(counter as u64)
}

/// Gets the current counter value for a client without incrementing.
pub fn get_counter(conn: &Connection, client_id: &str) -> Result<Option<u64>, LatticeError> {
    let result: Option<i64> = conn
        .query_row(
            "SELECT next_counter FROM client_counters WHERE client_id = ?",
            [client_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query counter for client {client_id}: {e}"),
        })?;

    debug!(client_id, ?result, "Retrieved counter");
    Ok(result.map(|c| c as u64))
}

/// Sets the counter value for a client.
///
/// Creates the record if it doesn't exist, otherwise updates it.
/// Use this for counter recovery when scanning existing documents.
pub fn set_counter(conn: &Connection, client_id: &str, value: u64) -> Result<(), LatticeError> {
    debug!(client_id, value, "Setting counter");

    conn.execute(
        "INSERT INTO client_counters (client_id, next_counter) VALUES (?1, ?2)
         ON CONFLICT(client_id) DO UPDATE SET next_counter = ?2",
        rusqlite::params![client_id, value as i64],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to set counter for client {client_id}: {e}"),
    })?;

    Ok(())
}

/// Sets the counter only if the new value is higher than the current value.
///
/// Useful for counter recovery to ensure we don't go backwards.
pub fn set_counter_if_higher(
    conn: &Connection,
    client_id: &str,
    value: u64,
) -> Result<(), LatticeError> {
    debug!(client_id, value, "Setting counter if higher");

    conn.execute(
        "INSERT INTO client_counters (client_id, next_counter) VALUES (?1, ?2)
         ON CONFLICT(client_id) DO UPDATE SET next_counter = MAX(next_counter, ?2)",
        rusqlite::params![client_id, value as i64],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to set counter for client {client_id}: {e}"),
    })?;

    Ok(())
}

/// Lists all client IDs with their current counter values.
pub fn list_all(conn: &Connection) -> Result<Vec<(String, u64)>, LatticeError> {
    let mut stmt = conn
        .prepare("SELECT client_id, next_counter FROM client_counters ORDER BY client_id")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare client counters query: {e}"),
        })?;

    let rows = stmt
        .query_map([], |row| {
            let client_id: String = row.get(0)?;
            let counter: i64 = row.get(1)?;
            Ok((client_id, counter as u64))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query client counters: {e}"),
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to read client counter row: {e}"),
        })?);
    }

    debug!(count = result.len(), "Listed all client counters");
    Ok(result)
}

/// Deletes a client's counter record.
pub fn delete(conn: &Connection, client_id: &str) -> Result<bool, LatticeError> {
    let rows_affected = conn
        .execute("DELETE FROM client_counters WHERE client_id = ?", [client_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete counter for client {client_id}: {e}"),
        })?;

    debug!(client_id, deleted = rows_affected > 0, "Deleted client counter");
    Ok(rows_affected > 0)
}
