use rusqlite::{Connection, Error, Row, params};
use tracing::{debug, info};

use crate::error::error_types::LatticeError;

/// The type of link between documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Link appears in the markdown body content.
    Body,
    /// Link from `blocked-by` frontmatter field (source is blocked by target).
    BlockedBy,
    /// Link from `blocking` frontmatter field (source blocks target).
    Blocking,
    /// Link from `discovered-from` frontmatter field.
    DiscoveredFrom,
}

/// A link record from the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRow {
    pub source_id: String,
    pub target_id: String,
    pub link_type: LinkType,
    pub position: u32,
}

/// Data for inserting a new link.
#[derive(Debug, Clone)]
pub struct InsertLink<'a> {
    pub source_id: &'a str,
    pub target_id: &'a str,
    pub link_type: LinkType,
    pub position: u32,
}

/// Inserts multiple links for a document in a single transaction.
pub fn insert_for_document(
    conn: &Connection,
    links: &[InsertLink<'_>],
) -> Result<(), LatticeError> {
    if links.is_empty() {
        return Ok(());
    }

    let source_id = links[0].source_id;
    debug!(source_id, count = links.len(), "Inserting links for document");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin transaction: {e}"),
    })?;

    for link in links {
        tx.execute(
            "INSERT INTO links (source_id, target_id, link_type, position)
             VALUES (?1, ?2, ?3, ?4)",
            params![link.source_id, link.target_id, link.link_type.as_str(), link.position],
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!(
                "Failed to insert link from {} to {}: {e}",
                link.source_id, link.target_id
            ),
        })?;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit link insertion: {e}"),
    })?;

    info!(source_id, count = links.len(), "Links inserted for document");
    Ok(())
}

/// Deletes all links originating from a document.
pub fn delete_by_source(conn: &Connection, source_id: &str) -> Result<usize, LatticeError> {
    debug!(source_id, "Deleting links from document");

    let rows_affected = conn
        .execute("DELETE FROM links WHERE source_id = ?", [source_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete links from {source_id}: {e}"),
        })?;

    if rows_affected > 0 {
        info!(source_id, deleted = rows_affected, "Links deleted from document");
    } else {
        debug!(source_id, "No links to delete from document");
    }

    Ok(rows_affected)
}

/// Deletes all links pointing to a document.
pub fn delete_by_target(conn: &Connection, target_id: &str) -> Result<usize, LatticeError> {
    debug!(target_id, "Deleting links to document");

    let rows_affected = conn
        .execute("DELETE FROM links WHERE target_id = ?", [target_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete links to {target_id}: {e}"),
        })?;

    if rows_affected > 0 {
        info!(target_id, deleted = rows_affected, "Links deleted to document");
    } else {
        debug!(target_id, "No links to delete to document");
    }

    Ok(rows_affected)
}

/// Deletes a specific link between two documents.
pub fn delete_by_source_and_target(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
) -> Result<usize, LatticeError> {
    debug!(source_id, target_id, "Deleting specific link");

    let rows_affected = conn
        .execute("DELETE FROM links WHERE source_id = ? AND target_id = ?", params![
            source_id, target_id
        ])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to delete link from {source_id} to {target_id}: {e}"),
        })?;

    if rows_affected > 0 {
        info!(source_id, target_id, deleted = rows_affected, "Link deleted");
    } else {
        debug!(source_id, target_id, "Link not found for deletion");
    }

    Ok(rows_affected)
}

/// Queries outgoing links from a document (links-from).
pub fn query_outgoing(conn: &Connection, source_id: &str) -> Result<Vec<LinkRow>, LatticeError> {
    debug!(source_id, "Querying outgoing links");

    let mut stmt = conn
        .prepare("SELECT * FROM links WHERE source_id = ? ORDER BY position")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare outgoing links query: {e}"),
        })?;

    let rows = stmt
        .query_map([source_id], row_to_link)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query outgoing links for {source_id}: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect outgoing links: {e}"),
        })?;

    debug!(source_id, count = rows.len(), "Outgoing links found");
    Ok(rows)
}

/// Queries outgoing links from a document filtered by link type.
pub fn query_outgoing_by_type(
    conn: &Connection,
    source_id: &str,
    link_type: LinkType,
) -> Result<Vec<LinkRow>, LatticeError> {
    debug!(source_id, link_type = %link_type, "Querying outgoing links by type");

    let mut stmt = conn
        .prepare("SELECT * FROM links WHERE source_id = ? AND link_type = ? ORDER BY position")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare outgoing links query: {e}"),
        })?;

    let rows = stmt
        .query_map(params![source_id, link_type.as_str()], row_to_link)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query outgoing links for {source_id}: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect outgoing links: {e}"),
        })?;

    debug!(source_id, link_type = %link_type, count = rows.len(), "Outgoing links found by type");
    Ok(rows)
}

/// Queries incoming links to a document (links-to/backlinks).
pub fn query_incoming(conn: &Connection, target_id: &str) -> Result<Vec<LinkRow>, LatticeError> {
    debug!(target_id, "Querying incoming links");

    let mut stmt = conn
        .prepare("SELECT * FROM links WHERE target_id = ? ORDER BY source_id, position")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare incoming links query: {e}"),
        })?;

    let rows = stmt
        .query_map([target_id], row_to_link)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query incoming links for {target_id}: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect incoming links: {e}"),
        })?;

    debug!(target_id, count = rows.len(), "Incoming links found");
    Ok(rows)
}

/// Queries incoming links to a document filtered by link type.
pub fn query_incoming_by_type(
    conn: &Connection,
    target_id: &str,
    link_type: LinkType,
) -> Result<Vec<LinkRow>, LatticeError> {
    debug!(target_id, link_type = %link_type, "Querying incoming links by type");

    let mut stmt = conn
        .prepare(
            "SELECT * FROM links WHERE target_id = ? AND link_type = ? ORDER BY source_id, position",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare incoming links query: {e}"),
        })?;

    let rows = stmt
        .query_map(params![target_id, link_type.as_str()], row_to_link)
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query incoming links for {target_id}: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect incoming links: {e}"),
        })?;

    debug!(target_id, link_type = %link_type, count = rows.len(), "Incoming links found by type");
    Ok(rows)
}

/// Checks if a link exists between two documents.
pub fn exists(conn: &Connection, source_id: &str, target_id: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM links WHERE source_id = ? AND target_id = ?",
            params![source_id, target_id],
            |row| row.get(0),
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check link existence: {e}"),
        })?;

    Ok(count > 0)
}

/// Counts outgoing links from a document.
pub fn count_outgoing(conn: &Connection, source_id: &str) -> Result<u64, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM links WHERE source_id = ?", [source_id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to count outgoing links: {e}"),
        })?;

    Ok(count as u64)
}

/// Counts incoming links to a document.
pub fn count_incoming(conn: &Connection, target_id: &str) -> Result<u64, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM links WHERE target_id = ?", [target_id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to count incoming links: {e}"),
        })?;

    Ok(count as u64)
}

/// Returns all source document IDs that have zero incoming links (orphans).
pub fn find_orphan_sources(conn: &Connection) -> Result<Vec<String>, LatticeError> {
    debug!("Finding orphan documents");

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT d.id FROM documents d
             WHERE NOT EXISTS (SELECT 1 FROM links l WHERE l.target_id = d.id)",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare orphan query: {e}"),
        })?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query orphan documents: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect orphan documents: {e}"),
        })?;

    debug!(count = rows.len(), "Orphan documents found");
    Ok(rows)
}

/// Returns all unique target IDs that a document links to.
pub fn get_target_ids(conn: &Connection, source_id: &str) -> Result<Vec<String>, LatticeError> {
    debug!(source_id, "Getting target IDs");

    let mut stmt = conn
        .prepare("SELECT DISTINCT target_id FROM links WHERE source_id = ? ORDER BY target_id")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare target IDs query: {e}"),
        })?;

    let rows = stmt
        .query_map([source_id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query target IDs for {source_id}: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect target IDs: {e}"),
        })?;

    debug!(source_id, count = rows.len(), "Target IDs found");
    Ok(rows)
}

/// Returns all unique source IDs that link to a document.
pub fn get_source_ids(conn: &Connection, target_id: &str) -> Result<Vec<String>, LatticeError> {
    debug!(target_id, "Getting source IDs");

    let mut stmt = conn
        .prepare("SELECT DISTINCT source_id FROM links WHERE target_id = ? ORDER BY source_id")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare source IDs query: {e}"),
        })?;

    let rows = stmt
        .query_map([target_id], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query source IDs for {target_id}: {e}"),
        })?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect source IDs: {e}"),
        })?;

    debug!(target_id, count = rows.len(), "Source IDs found");
    Ok(rows)
}

impl LinkType {
    fn as_str(&self) -> &'static str {
        match self {
            LinkType::Body => "body",
            LinkType::BlockedBy => "blocked_by",
            LinkType::Blocking => "blocking",
            LinkType::DiscoveredFrom => "discovered_from",
        }
    }

    fn from_str(s: &str) -> Option<LinkType> {
        match s {
            "body" => Some(LinkType::Body),
            "blocked_by" => Some(LinkType::BlockedBy),
            "blocking" => Some(LinkType::Blocking),
            "discovered_from" => Some(LinkType::DiscoveredFrom),
            _ => None,
        }
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn row_to_link(row: &Row<'_>) -> Result<LinkRow, Error> {
    let link_type_str: String = row.get("link_type")?;
    let link_type = LinkType::from_str(&link_type_str).unwrap_or_else(|| {
        panic!("Invalid link_type in database: {link_type_str}. Database may be corrupted.")
    });

    Ok(LinkRow {
        source_id: row.get("source_id")?,
        target_id: row.get("target_id")?,
        link_type,
        position: row.get("position")?,
    })
}
