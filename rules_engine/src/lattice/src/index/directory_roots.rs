use rusqlite::{Connection, OptionalExtension};
use tracing::debug;

use crate::error::error_types::LatticeError;

/// A root document's directory entry in the hierarchy.
#[derive(Debug, Clone)]
pub struct DirectoryRoot {
    pub directory_path: String,
    pub root_id: String,
    pub parent_path: Option<String>,
    pub depth: u32,
}

/// Inserts or updates a directory root entry.
pub fn upsert(conn: &Connection, root: &DirectoryRoot) -> Result<(), LatticeError> {
    debug!(
        directory_path = root.directory_path,
        root_id = root.root_id,
        depth = root.depth,
        "Upserting directory root"
    );

    conn.execute(
        "INSERT INTO directory_roots (directory_path, root_id, parent_path, depth)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(directory_path) DO UPDATE SET
             root_id = ?2, parent_path = ?3, depth = ?4",
        rusqlite::params![root.directory_path, root.root_id, root.parent_path, root.depth],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to upsert directory root {}: {e}", root.directory_path),
    })?;

    Ok(())
}

/// Gets the root document ID for a directory.
pub fn get_root_id(
    conn: &Connection,
    directory_path: &str,
) -> Result<Option<String>, LatticeError> {
    let result: Option<String> = conn
        .query_row(
            "SELECT root_id FROM directory_roots WHERE directory_path = ?",
            [directory_path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query root for directory {directory_path}: {e}"),
        })?;

    debug!(directory_path, ?result, "Retrieved directory root ID");
    Ok(result)
}

/// Gets the full directory root entry.
pub fn get(conn: &Connection, directory_path: &str) -> Result<Option<DirectoryRoot>, LatticeError> {
    let result = conn
        .query_row(
            "SELECT directory_path, root_id, parent_path, depth FROM directory_roots WHERE directory_path = ?",
            [directory_path],
            |row| {
                Ok(DirectoryRoot {
                    directory_path: row.get(0)?,
                    root_id: row.get(1)?,
                    parent_path: row.get(2)?,
                    depth: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query directory root {directory_path}: {e}"),
        })?;

    debug!(directory_path, found = result.is_some(), "Retrieved directory root");
    Ok(result)
}

/// Gets the ancestor chain for a directory, including the directory itself.
///
/// Returns entries ordered from root (depth 0) to the queried directory.
/// This is designed for template composition where you need to walk from the
/// root down to the current directory collecting context from each level.
///
/// Returns an empty vector if the directory is not in the hierarchy.
pub fn get_ancestors(
    conn: &Connection,
    directory_path: &str,
) -> Result<Vec<DirectoryRoot>, LatticeError> {
    let mut ancestors = Vec::new();
    let mut current_path = Some(directory_path.to_string());

    while let Some(path) = current_path {
        if let Some(root) = get(conn, &path)? {
            current_path = root.parent_path.clone();
            ancestors.push(root);
        } else {
            break;
        }
    }

    ancestors.reverse();
    debug!(directory_path, count = ancestors.len(), "Retrieved ancestors");
    Ok(ancestors)
}

/// Gets immediate children directories of a parent directory.
pub fn get_children(
    conn: &Connection,
    parent_path: &str,
) -> Result<Vec<DirectoryRoot>, LatticeError> {
    let mut stmt = conn
        .prepare(
            "SELECT directory_path, root_id, parent_path, depth
             FROM directory_roots
             WHERE parent_path = ?
             ORDER BY directory_path",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare children query: {e}"),
        })?;

    let rows = stmt
        .query_map([parent_path], |row| {
            Ok(DirectoryRoot {
                directory_path: row.get(0)?,
                root_id: row.get(1)?,
                parent_path: row.get(2)?,
                depth: row.get(3)?,
            })
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query children of {parent_path}: {e}"),
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to read directory root row: {e}"),
        })?);
    }

    debug!(parent_path, count = result.len(), "Retrieved child directories");
    Ok(result)
}

/// Lists all directory roots at a specific depth.
pub fn list_at_depth(conn: &Connection, depth: u32) -> Result<Vec<DirectoryRoot>, LatticeError> {
    let mut stmt = conn
        .prepare(
            "SELECT directory_path, root_id, parent_path, depth
             FROM directory_roots
             WHERE depth = ?
             ORDER BY directory_path",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare depth query: {e}"),
        })?;

    let rows = stmt
        .query_map([depth], |row| {
            Ok(DirectoryRoot {
                directory_path: row.get(0)?,
                root_id: row.get(1)?,
                parent_path: row.get(2)?,
                depth: row.get(3)?,
            })
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query roots at depth {depth}: {e}"),
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to read directory root row: {e}"),
        })?);
    }

    debug!(depth, count = result.len(), "Listed roots at depth");
    Ok(result)
}

/// Lists all directory roots, ordered by depth then path.
pub fn list_all(conn: &Connection) -> Result<Vec<DirectoryRoot>, LatticeError> {
    let mut stmt = conn
        .prepare(
            "SELECT directory_path, root_id, parent_path, depth
             FROM directory_roots
             ORDER BY depth, directory_path",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare list query: {e}"),
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok(DirectoryRoot {
                directory_path: row.get(0)?,
                root_id: row.get(1)?,
                parent_path: row.get(2)?,
                depth: row.get(3)?,
            })
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query all directory roots: {e}"),
        })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to read directory root row: {e}"),
        })?);
    }

    debug!(count = result.len(), "Listed all directory roots");
    Ok(result)
}

/// Deletes a directory root entry.
pub fn delete(conn: &Connection, directory_path: &str) -> Result<bool, LatticeError> {
    let rows_affected = conn
        .execute("DELETE FROM directory_roots WHERE directory_path = ?", [directory_path])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete directory root {directory_path}: {e}"),
        })?;

    debug!(directory_path, deleted = rows_affected > 0, "Deleted directory root");
    Ok(rows_affected > 0)
}

/// Clears all directory root entries.
///
/// Used during full index rebuild.
pub fn clear_all(conn: &Connection) -> Result<usize, LatticeError> {
    let rows_affected = conn.execute("DELETE FROM directory_roots", []).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to clear directory roots: {e}") }
    })?;

    debug!(count = rows_affected, "Cleared all directory roots");
    Ok(rows_affected)
}
