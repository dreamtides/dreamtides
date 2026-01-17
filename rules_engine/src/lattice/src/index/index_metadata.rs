use chrono::{DateTime, Utc};
use rusqlite::Connection;
use tracing::debug;

use crate::error::error_types::LatticeError;

/// Metadata about the index state.
#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub schema_version: u32,
    pub last_commit: Option<String>,
    pub last_indexed: DateTime<Utc>,
}

/// Retrieves the current index metadata.
pub fn get_metadata(conn: &Connection) -> Result<Option<IndexMetadata>, LatticeError> {
    let result = conn.query_row(
        "SELECT schema_version, last_commit, last_indexed FROM index_metadata WHERE id = 1",
        [],
        |row| {
            let schema_version: u32 = row.get(0)?;
            let last_commit: Option<String> = row.get(1)?;
            let last_indexed_str: String = row.get(2)?;
            Ok((schema_version, last_commit, last_indexed_str))
        },
    );

    match result {
        Ok((schema_version, last_commit, last_indexed_str)) => {
            let last_indexed = DateTime::parse_from_rfc3339(&last_indexed_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            debug!(schema_version, ?last_commit, "Retrieved index metadata");
            Ok(Some(IndexMetadata { schema_version, last_commit, last_indexed }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            debug!("No index metadata found");
            Ok(None)
        }
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg))) if msg.contains("no such table") => {
            debug!("index_metadata table does not exist");
            Ok(None)
        }
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to read index metadata: {e}"),
        }),
    }
}

/// Gets the last indexed git commit hash.
pub fn get_last_commit(conn: &Connection) -> Result<Option<String>, LatticeError> {
    let result: Result<Option<String>, _> =
        conn.query_row("SELECT last_commit FROM index_metadata WHERE id = 1", [], |row| row.get(0));

    match result {
        Ok(commit) => {
            debug!(?commit, "Retrieved last indexed commit");
            Ok(commit)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg))) if msg.contains("no such table") => {
            Ok(None)
        }
        Err(e) => {
            Err(LatticeError::DatabaseError { reason: format!("Failed to read last commit: {e}") })
        }
    }
}

/// Gets the last indexed timestamp.
pub fn get_last_indexed(conn: &Connection) -> Result<Option<DateTime<Utc>>, LatticeError> {
    let result: Result<String, _> =
        conn.query_row("SELECT last_indexed FROM index_metadata WHERE id = 1", [], |row| {
            row.get(0)
        });

    match result {
        Ok(timestamp_str) => {
            let dt =
                DateTime::parse_from_rfc3339(&timestamp_str).map(|dt| dt.with_timezone(&Utc)).ok();
            debug!(?dt, "Retrieved last indexed timestamp");
            Ok(dt)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg))) if msg.contains("no such table") => {
            Ok(None)
        }
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to read last indexed timestamp: {e}"),
        }),
    }
}

/// Updates the last indexed commit hash.
pub fn set_last_commit(conn: &Connection, commit: Option<&str>) -> Result<(), LatticeError> {
    let timestamp = Utc::now().to_rfc3339();
    debug!(?commit, %timestamp, "Updating last indexed commit");

    conn.execute(
        "UPDATE index_metadata SET last_commit = ?1, last_indexed = ?2 WHERE id = 1",
        rusqlite::params![commit, timestamp],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to update last commit: {e}"),
    })?;

    Ok(())
}

/// Updates the last indexed timestamp to the current time.
pub fn touch_last_indexed(conn: &Connection) -> Result<(), LatticeError> {
    let timestamp = Utc::now().to_rfc3339();
    debug!(%timestamp, "Updating last indexed timestamp");

    conn.execute("UPDATE index_metadata SET last_indexed = ?1 WHERE id = 1", [&timestamp])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to update last indexed: {e}"),
        })?;

    Ok(())
}
