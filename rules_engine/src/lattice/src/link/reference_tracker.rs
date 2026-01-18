use rusqlite::Connection;
use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::index::document_types::DocumentRow;
use crate::index::link_queries::{LinkRow, LinkType};
use crate::index::{document_queries, link_queries};

/// A reference with full document metadata.
///
/// This is the high-level result type for reference queries, combining the raw
/// link data from the index with the full document metadata for the referenced
/// document. This enables callers to display rich information about references
/// without additional lookups.
#[derive(Debug, Clone)]
pub struct Reference {
    /// The link type (body, blocking, blocked_by, discovered_from).
    pub link_type: LinkType,
    /// Position of the link within the source document.
    pub position: u32,
    /// Full metadata for the referenced document.
    pub document: DocumentRow,
}

/// Query result for forward references (links from a document).
#[derive(Debug, Clone)]
pub struct ForwardReferences {
    /// The source document's Lattice ID.
    pub source_id: String,
    /// All documents this source links to, with metadata.
    pub references: Vec<Reference>,
}

/// Query result for reverse references (backlinks to a document).
#[derive(Debug, Clone)]
pub struct ReverseReferences {
    /// The target document's Lattice ID.
    pub target_id: String,
    /// All documents that link to this target, with metadata.
    pub references: Vec<Reference>,
}

/// Queries forward references: documents that a source document links to.
///
/// Returns all outgoing links from the specified document, enriched with full
/// document metadata for each target. This is used by `lat links-from` to show
/// what a document references.
///
/// # Arguments
///
/// * `conn` - Database connection to the index
/// * `source_id` - The Lattice ID of the document to query links from
///
/// # Returns
///
/// A `ForwardReferences` struct containing the source ID and all referenced
/// documents with their metadata. Documents that cannot be found in the index
/// are silently skipped (this can happen if the index is out of sync).
pub fn query_forward(
    conn: &Connection,
    source_id: &str,
) -> Result<ForwardReferences, LatticeError> {
    debug!(source_id, "Querying forward references");
    let link_rows = link_queries::query_outgoing(conn, source_id)?;
    let references = enrich_target_links(conn, &link_rows)?;
    debug!(source_id, count = references.len(), "Forward references found");
    Ok(ForwardReferences { source_id: source_id.to_string(), references })
}

/// Queries forward references filtered by link type.
///
/// Same as `query_forward` but only returns links of the specified type.
/// Useful for querying only body links or only dependency relationships.
pub fn query_forward_by_type(
    conn: &Connection,
    source_id: &str,
    link_type: LinkType,
) -> Result<ForwardReferences, LatticeError> {
    debug!(source_id, link_type = %link_type, "Querying forward references by type");
    let link_rows = link_queries::query_outgoing_by_type(conn, source_id, link_type)?;
    let references = enrich_target_links(conn, &link_rows)?;
    debug!(
        source_id,
        link_type = %link_type,
        count = references.len(),
        "Forward references found by type"
    );
    Ok(ForwardReferences { source_id: source_id.to_string(), references })
}

/// Queries reverse references (backlinks): documents that link to a target.
///
/// Returns all incoming links to the specified document, enriched with full
/// document metadata for each source. This is used by `lat links-to` to show
/// backlinks and by `lat impact` to analyze affected documents.
///
/// # Arguments
///
/// * `conn` - Database connection to the index
/// * `target_id` - The Lattice ID of the document to query backlinks for
///
/// # Returns
///
/// A `ReverseReferences` struct containing the target ID and all referencing
/// documents with their metadata. Documents that cannot be found in the index
/// are silently skipped (this can happen if the index is out of sync).
pub fn query_reverse(
    conn: &Connection,
    target_id: &str,
) -> Result<ReverseReferences, LatticeError> {
    debug!(target_id, "Querying reverse references (backlinks)");
    let link_rows = link_queries::query_incoming(conn, target_id)?;
    let references = enrich_source_links(conn, &link_rows)?;
    debug!(target_id, count = references.len(), "Reverse references found");
    Ok(ReverseReferences { target_id: target_id.to_string(), references })
}

/// Queries reverse references filtered by link type.
///
/// Same as `query_reverse` but only returns links of the specified type.
/// Useful for finding all documents that depend on this one (blocked_by)
/// or all documents this one blocks.
pub fn query_reverse_by_type(
    conn: &Connection,
    target_id: &str,
    link_type: LinkType,
) -> Result<ReverseReferences, LatticeError> {
    debug!(target_id, link_type = %link_type, "Querying reverse references by type");
    let link_rows = link_queries::query_incoming_by_type(conn, target_id, link_type)?;
    let references = enrich_source_links(conn, &link_rows)?;
    debug!(
        target_id,
        link_type = %link_type,
        count = references.len(),
        "Reverse references found by type"
    );
    Ok(ReverseReferences { target_id: target_id.to_string(), references })
}

/// Finds orphan documents: those with no incoming links.
///
/// Returns documents that are not referenced by any other document. This is
/// used by `lat orphans` to identify disconnected documents that may need
/// to be linked or removed.
///
/// # Arguments
///
/// * `conn` - Database connection to the index
///
/// # Returns
///
/// A vector of orphan documents with their full metadata. Root documents are
/// included by default; callers should filter if needed.
pub fn find_orphans(conn: &Connection) -> Result<Vec<DocumentRow>, LatticeError> {
    debug!("Finding orphan documents");
    let orphan_ids = link_queries::find_orphan_sources(conn)?;
    let mut orphans = Vec::with_capacity(orphan_ids.len());
    for id in &orphan_ids {
        if let Some(doc) = document_queries::lookup_by_id(conn, id)? {
            orphans.push(doc);
        }
    }
    debug!(count = orphans.len(), "Orphan documents found");
    Ok(orphans)
}

/// Enriches link rows with target document metadata.
///
/// For each link row, looks up the target document in the index and creates
/// a Reference with full metadata. Links to missing documents are skipped.
fn enrich_target_links(
    conn: &Connection,
    link_rows: &[LinkRow],
) -> Result<Vec<Reference>, LatticeError> {
    let mut references = Vec::with_capacity(link_rows.len());
    for link in link_rows {
        if let Some(doc) = document_queries::lookup_by_id(conn, &link.target_id)? {
            references.push(Reference {
                link_type: link.link_type,
                position: link.position,
                document: doc,
            });
        }
    }
    Ok(references)
}

/// Enriches link rows with source document metadata.
///
/// For each link row, looks up the source document in the index and creates
/// a Reference with full metadata. Links from missing documents are skipped.
fn enrich_source_links(
    conn: &Connection,
    link_rows: &[LinkRow],
) -> Result<Vec<Reference>, LatticeError> {
    let mut references = Vec::with_capacity(link_rows.len());
    for link in link_rows {
        if let Some(doc) = document_queries::lookup_by_id(conn, &link.source_id)? {
            references.push(Reference {
                link_type: link.link_type,
                position: link.position,
                document: doc,
            });
        }
    }
    Ok(references)
}
