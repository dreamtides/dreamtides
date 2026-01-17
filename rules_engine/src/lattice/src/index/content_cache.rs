use std::fs;
use std::path::Path;
use std::time::SystemTime;

use chrono::Utc;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use tracing::{debug, info};

use crate::error::error_types::LatticeError;
use crate::index::schema_definition::CONTENT_CACHE_MAX_ENTRIES;

/// Data returned from the content cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedContent {
    pub document_id: String,
    pub content: String,
    pub content_hash: String,
}

/// Statistics about the content cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    pub entry_count: u64,
    pub total_content_bytes: u64,
}

/// Retrieves cached content for a document if the cache entry is still valid.
///
/// Validates the entry by comparing the stored `file_mtime` against the current
/// filesystem mtime. Returns `None` if:
/// - No cache entry exists for the document
/// - The cache entry is stale (file has been modified)
/// - The file cannot be accessed to check mtime
///
/// This is a read-through cache operation; callers should update the cache
/// via `put_content` when this returns `None`.
pub fn get_cached_content(
    conn: &Connection,
    document_id: &str,
    file_path: &Path,
) -> Result<Option<CachedContent>, LatticeError> {
    debug!(document_id, path = %file_path.display(), "Checking content cache");

    let result = conn.query_row(
        "SELECT content, content_hash, file_mtime FROM content_cache WHERE document_id = ?",
        [document_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
    );

    match result {
        Ok((content, content_hash, cached_mtime)) => {
            let current_mtime = get_file_mtime(file_path);
            if current_mtime == cached_mtime as u64 {
                debug!(document_id, "Content cache hit");
                update_accessed_at(conn, document_id)?;
                Ok(Some(CachedContent {
                    document_id: document_id.to_string(),
                    content,
                    content_hash,
                }))
            } else {
                debug!(
                    document_id,
                    cached_mtime, current_mtime, "Content cache stale (mtime changed)"
                );
                Ok(None)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            debug!(document_id, "Content cache miss");
            Ok(None)
        }
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to query content cache for {document_id}: {e}"),
        }),
    }
}

/// Stores content in the cache for a document.
///
/// If an entry already exists for the document, it is replaced. The content
/// hash is computed as SHA-256 of the content. The `file_mtime` should be
/// obtained from the filesystem at the time the content was read.
///
/// After inserting, triggers eviction if the cache exceeds the maximum size.
pub fn put_content(
    conn: &Connection,
    document_id: &str,
    content: &str,
    file_mtime: u64,
) -> Result<(), LatticeError> {
    let content_hash = compute_content_hash(content);
    let timestamp = Utc::now().to_rfc3339();
    debug!(document_id, content_len = content.len(), "Caching document content");

    conn.execute(
        "INSERT INTO content_cache (document_id, content, content_hash, accessed_at, file_mtime)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(document_id) DO UPDATE SET
            content = excluded.content,
            content_hash = excluded.content_hash,
            accessed_at = excluded.accessed_at,
            file_mtime = excluded.file_mtime",
        params![document_id, content, content_hash, timestamp, file_mtime as i64],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to cache content for {document_id}: {e}"),
    })?;

    evict_if_needed(conn)?;
    Ok(())
}

/// Invalidates (removes) the cache entry for a specific document.
///
/// Returns `true` if an entry was removed, `false` if no entry existed.
pub fn invalidate_cache(conn: &Connection, document_id: &str) -> Result<bool, LatticeError> {
    debug!(document_id, "Invalidating content cache entry");

    let rows_deleted = conn
        .execute("DELETE FROM content_cache WHERE document_id = ?", [document_id])
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to invalidate cache for {document_id}: {e}"),
        })?;

    if rows_deleted > 0 {
        info!(document_id, "Content cache entry invalidated");
        Ok(true)
    } else {
        debug!(document_id, "No cache entry to invalidate");
        Ok(false)
    }
}

/// Clears all entries from the content cache.
///
/// Returns the number of entries that were removed.
pub fn clear_cache(conn: &Connection) -> Result<u64, LatticeError> {
    debug!("Clearing entire content cache");

    let rows_deleted = conn.execute("DELETE FROM content_cache", []).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to clear content cache: {e}") }
    })?;

    info!(entries_cleared = rows_deleted, "Content cache cleared");
    Ok(rows_deleted as u64)
}

/// Returns statistics about the content cache.
pub fn get_cache_stats(conn: &Connection) -> Result<CacheStats, LatticeError> {
    debug!("Getting content cache statistics");

    let (entry_count, total_bytes): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(LENGTH(content)), 0) FROM content_cache",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to get content cache stats: {e}"),
        })?;

    Ok(CacheStats { entry_count: entry_count as u64, total_content_bytes: total_bytes as u64 })
}

/// Evicts the least-recently-accessed entries if the cache exceeds the maximum
/// size.
///
/// The eviction policy removes entries with the oldest `accessed_at` timestamps
/// until the cache is at or below the maximum entry count.
fn evict_if_needed(conn: &Connection) -> Result<(), LatticeError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM content_cache", [], |row| row.get(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to count cache entries: {e}"),
        })?;

    let max_entries = CONTENT_CACHE_MAX_ENTRIES as i64;
    if count <= max_entries {
        return Ok(());
    }

    let to_evict = count - max_entries;
    debug!(
        current_count = count,
        max = max_entries,
        evicting = to_evict,
        "Evicting old cache entries"
    );

    conn.execute(
        "DELETE FROM content_cache WHERE document_id IN (
            SELECT document_id FROM content_cache
            ORDER BY accessed_at ASC
            LIMIT ?
        )",
        [to_evict],
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to evict cache entries: {e}"),
    })?;

    info!(evicted = to_evict, "Cache entries evicted");
    Ok(())
}

/// Updates the accessed_at timestamp for a cache entry.
fn update_accessed_at(conn: &Connection, document_id: &str) -> Result<(), LatticeError> {
    let timestamp = Utc::now().to_rfc3339();
    conn.execute("UPDATE content_cache SET accessed_at = ? WHERE document_id = ?", params![
        timestamp,
        document_id
    ])
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to update accessed_at for {document_id}: {e}"),
    })?;
    Ok(())
}

/// Gets the modification time of a file as seconds since Unix epoch.
///
/// Returns 0 if the file doesn't exist or its mtime cannot be determined.
fn get_file_mtime(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

/// Computes the SHA-256 hash of the content as a hex string.
fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
