use rusqlite::{Connection, params};
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;

/// A single result from a full-text search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The Lattice ID of the matching document.
    pub document_id: String,
    /// BM25 relevance rank (lower is more relevant, typically negative).
    pub rank: f64,
}

/// A search result with an extracted snippet showing the match context.
#[derive(Debug, Clone)]
pub struct SearchResultWithSnippet {
    /// The Lattice ID of the matching document.
    pub document_id: String,
    /// BM25 relevance rank (lower is more relevant, typically negative).
    pub rank: f64,
    /// Snippet with match context, using markers for highlighting.
    pub snippet: String,
}

/// Indexes a document's body content for full-text search.
///
/// If the document already exists in the FTS index, it is replaced.
pub fn index_document(
    conn: &Connection,
    document_id: &str,
    body: &str,
) -> Result<(), LatticeError> {
    debug!(document_id, body_len = body.len(), "Indexing document for full-text search");

    // Delete existing entry if present, then insert new content.
    // Using a transaction ensures atomicity.
    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin FTS transaction: {e}"),
    })?;

    tx.execute("DELETE FROM fts_content WHERE document_id = ?", [document_id]).map_err(|e| {
        LatticeError::DatabaseError {
            reason: format!("Failed to delete old FTS content for {document_id}: {e}"),
        }
    })?;

    tx.execute("INSERT INTO fts_content (document_id, body) VALUES (?1, ?2)", params![
        document_id,
        body
    ])
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to insert FTS content for {document_id}: {e}"),
    })?;

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit FTS transaction: {e}"),
    })?;

    debug!(document_id, "Document indexed for full-text search");
    Ok(())
}

/// Indexes multiple documents in a single transaction for efficiency.
///
/// Each tuple is (document_id, body). Existing documents are replaced.
pub fn index_batch(conn: &Connection, documents: &[(&str, &str)]) -> Result<usize, LatticeError> {
    if documents.is_empty() {
        return Ok(0);
    }

    debug!(count = documents.len(), "Batch indexing documents for full-text search");

    let tx = conn.unchecked_transaction().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to begin FTS batch transaction: {e}"),
    })?;

    for (document_id, body) in documents {
        tx.execute("DELETE FROM fts_content WHERE document_id = ?", [document_id]).map_err(
            |e| LatticeError::DatabaseError {
                reason: format!("Failed to delete old FTS content for {document_id}: {e}"),
            },
        )?;

        tx.execute("INSERT INTO fts_content (document_id, body) VALUES (?1, ?2)", params![
            document_id,
            body
        ])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to insert FTS content for {document_id}: {e}"),
        })?;
    }

    tx.commit().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to commit FTS batch transaction: {e}"),
    })?;

    info!(count = documents.len(), "Batch FTS indexing complete");
    Ok(documents.len())
}

/// Removes a document from the full-text search index.
///
/// Returns true if the document was removed, false if it wasn't in the index.
pub fn remove_document(conn: &Connection, document_id: &str) -> Result<bool, LatticeError> {
    debug!(document_id, "Removing document from FTS index");

    let rows = conn
        .execute("DELETE FROM fts_content WHERE document_id = ?", [document_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to remove {document_id} from FTS: {e}"),
        })?;

    if rows > 0 {
        debug!(document_id, "Document removed from FTS index");
        Ok(true)
    } else {
        debug!(document_id, "Document not found in FTS index");
        Ok(false)
    }
}

/// Executes a full-text search query and returns results ranked by relevance.
///
/// Supports FTS5 query syntax:
/// - Word: `error` - documents containing "error"
/// - Phrase: `"login bug"` - exact phrase match
/// - AND: `error AND login` - both terms
/// - OR: `error OR warning` - either term
/// - NOT: `error NOT test` - first without second
/// - Prefix: `auth*` - words starting with "auth"
/// - NEAR: `NEAR(error login, 5)` - terms within 5 words
///
/// Results are ordered by BM25 rank (lower values = more relevant).
pub fn search(conn: &Connection, query: &str) -> Result<Vec<SearchResult>, LatticeError> {
    search_with_limit(conn, query, None)
}

/// Executes a full-text search with an optional result limit.
pub fn search_with_limit(
    conn: &Connection,
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, LatticeError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    debug!(query, limit = ?limit, "Executing FTS search");

    let sql = match limit {
        Some(n) => format!(
            "SELECT document_id, bm25(fts_content) as rank
             FROM fts_content
             WHERE fts_content MATCH ?1
             ORDER BY rank
             LIMIT {n}"
        ),
        None => "SELECT document_id, bm25(fts_content) as rank
                 FROM fts_content
                 WHERE fts_content MATCH ?1
                 ORDER BY rank"
            .to_string(),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare FTS search: {e}"),
    })?;

    let results = stmt
        .query_map([query], |row| Ok(SearchResult { document_id: row.get(0)?, rank: row.get(1)? }))
        .map_err(|e| {
            warn!(query, error = %e, "FTS search query failed");
            LatticeError::DatabaseError { reason: format!("FTS search failed for '{query}': {e}") }
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect FTS results: {e}"),
        })?;

    debug!(query, count = results.len(), "FTS search returned results");
    Ok(results)
}

/// Executes a full-text search and returns results with snippet context.
///
/// Snippets show the matching text with surrounding context. The `highlight`
/// markers are used to wrap matching terms (e.g., `<b>` and `</b>`).
pub fn search_with_snippets(
    conn: &Connection,
    query: &str,
    highlight_start: &str,
    highlight_end: &str,
    limit: Option<u32>,
) -> Result<Vec<SearchResultWithSnippet>, LatticeError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    debug!(query, limit = ?limit, "Executing FTS search with snippets");

    // snippet(fts_content, column_idx, start_mark, end_mark, ellipsis, max_tokens)
    // Column 1 is 'body' (0 is document_id which is unindexed)
    let sql = match limit {
        Some(n) => format!(
            "SELECT document_id, bm25(fts_content) as rank,
                    snippet(fts_content, 1, ?2, ?3, '...', 32) as snippet
             FROM fts_content
             WHERE fts_content MATCH ?1
             ORDER BY rank
             LIMIT {n}"
        ),
        None => "SELECT document_id, bm25(fts_content) as rank,
                        snippet(fts_content, 1, ?2, ?3, '...', 32) as snippet
                 FROM fts_content
                 WHERE fts_content MATCH ?1
                 ORDER BY rank"
            .to_string(),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to prepare FTS snippet search: {e}"),
    })?;

    let results = stmt
        .query_map(params![query, highlight_start, highlight_end], |row| {
            Ok(SearchResultWithSnippet {
                document_id: row.get(0)?,
                rank: row.get(1)?,
                snippet: row.get(2)?,
            })
        })
        .map_err(|e| {
            warn!(query, error = %e, "FTS snippet search failed");
            LatticeError::DatabaseError {
                reason: format!("FTS snippet search failed for '{query}': {e}"),
            }
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect FTS snippet results: {e}"),
        })?;

    debug!(query, count = results.len(), "FTS snippet search returned results");
    Ok(results)
}

/// Clears all content from the FTS index.
///
/// This is typically called before a full rebuild.
pub fn clear_index(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Clearing FTS index");

    conn.execute("DELETE FROM fts_content", []).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to clear FTS index: {e}"),
    })?;

    info!("FTS index cleared");
    Ok(())
}

/// Returns the number of documents in the FTS index.
pub fn count(conn: &Connection) -> Result<u64, LatticeError> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM fts_content", [], |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError { reason: format!("Failed to count FTS documents: {e}") }
        })?;

    Ok(count as u64)
}

/// Checks if a document exists in the FTS index.
pub fn exists(conn: &Connection, document_id: &str) -> Result<bool, LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM fts_content WHERE document_id = ?", [document_id], |row| {
            row.get(0)
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to check FTS existence for {document_id}: {e}"),
        })?;

    Ok(count > 0)
}
