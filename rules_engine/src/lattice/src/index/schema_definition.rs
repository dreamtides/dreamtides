use rusqlite::Connection;
use tracing::{debug, info};

use crate::error::error_types::LatticeError;

/// Current schema version. Increment when schema changes require rebuild.
pub const SCHEMA_VERSION: u32 = 2;
/// Maximum number of documents to keep in the content cache.
pub const CONTENT_CACHE_MAX_ENTRIES: u32 = 100;

/// Creates all Lattice index tables, indices, and triggers.
///
/// This function should be called once when initializing a new index database.
/// If the schema already exists, this will fail. Use [`schema_version`] to
/// check for version mismatches that require a full rebuild.
pub fn create_schema(conn: &Connection) -> Result<(), LatticeError> {
    info!("Creating Lattice index schema version {SCHEMA_VERSION}");

    create_documents_table(conn)?;
    create_links_table(conn)?;
    create_labels_table(conn)?;
    create_index_metadata_table(conn)?;
    create_client_counters_table(conn)?;
    create_directory_roots_table(conn)?;
    create_content_cache_table(conn)?;
    create_views_table(conn)?;
    create_fts_table(conn)?;
    create_link_count_triggers(conn)?;
    create_view_count_triggers(conn)?;
    create_fts_triggers(conn)?;

    info!("Schema creation complete");
    Ok(())
}

/// Returns the schema version stored in the index, or None if the table doesn't
/// exist.
pub fn schema_version(conn: &Connection) -> Result<Option<u32>, LatticeError> {
    let result: Result<u32, _> =
        conn.query_row("SELECT schema_version FROM index_metadata WHERE id = 1", [], |row| {
            row.get(0)
        });

    match result {
        Ok(version) => {
            debug!("Index schema version: {version}");
            Ok(Some(version))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            debug!("No schema version found (empty index_metadata)");
            Ok(None)
        }
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg))) if msg.contains("no such table") => {
            debug!("index_metadata table does not exist");
            Ok(None)
        }
        Err(e) => Err(LatticeError::DatabaseError {
            reason: format!("Failed to read schema version: {e}"),
        }),
    }
}

/// Checks if the current schema version matches the expected version.
pub fn schema_is_current(conn: &Connection) -> Result<bool, LatticeError> {
    match schema_version(conn)? {
        Some(version) => Ok(version == SCHEMA_VERSION),
        None => Ok(false),
    }
}

/// Runs FTS5 optimization after bulk operations.
pub fn optimize_fts(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Optimizing FTS5 index");
    conn.execute("INSERT INTO fts_content(fts_content) VALUES('optimize')", []).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to optimize FTS index: {e}") }
    })?;
    Ok(())
}

fn create_documents_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating documents table");
    conn.execute_batch(
        "
        CREATE TABLE documents (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            task_type TEXT,
            is_closed INTEGER NOT NULL DEFAULT 0,
            priority INTEGER,
            created_at TEXT,
            updated_at TEXT,
            closed_at TEXT,
            body_hash TEXT NOT NULL,
            indexed_at TEXT NOT NULL,
            content_length INTEGER NOT NULL,
            link_count INTEGER NOT NULL DEFAULT 0,
            backlink_count INTEGER NOT NULL DEFAULT 0,
            view_count INTEGER NOT NULL DEFAULT 0,
            is_root INTEGER NOT NULL DEFAULT 0,
            in_tasks_dir INTEGER NOT NULL DEFAULT 0,
            in_docs_dir INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX idx_documents_path ON documents(path);
        CREATE INDEX idx_documents_name ON documents(name);
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create documents table: {e}"),
    })
}

fn create_links_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating links table");
    conn.execute_batch(
        "
        CREATE TABLE links (
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            link_type TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (source_id, target_id, position)
        );

        CREATE INDEX idx_links_source ON links(source_id);
        CREATE INDEX idx_links_target ON links(target_id);
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create links table: {e}"),
    })
}

fn create_labels_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating labels table");
    conn.execute_batch(
        "
        CREATE TABLE labels (
            document_id TEXT NOT NULL,
            label TEXT NOT NULL,
            PRIMARY KEY (document_id, label)
        );

        CREATE INDEX idx_labels_document ON labels(document_id);
        CREATE INDEX idx_labels_label ON labels(label);
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create labels table: {e}"),
    })
}

fn create_index_metadata_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating index_metadata table");
    // Use strftime to produce RFC3339-compatible format for timestamp parsing
    conn.execute_batch(&format!(
        "
        CREATE TABLE index_metadata (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL,
            last_commit TEXT,
            last_indexed TEXT NOT NULL
        );

        INSERT INTO index_metadata (id, schema_version, last_indexed)
        VALUES (1, {SCHEMA_VERSION}, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
        "
    ))
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create index_metadata table: {e}"),
    })
}

fn create_client_counters_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating client_counters table");
    conn.execute_batch(
        "
        CREATE TABLE client_counters (
            client_id TEXT PRIMARY KEY,
            next_counter INTEGER NOT NULL DEFAULT 0
        );
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create client_counters table: {e}"),
    })
}

fn create_directory_roots_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating directory_roots table");
    conn.execute_batch(
        "
        CREATE TABLE directory_roots (
            directory_path TEXT PRIMARY KEY,
            root_id TEXT NOT NULL,
            parent_path TEXT,
            depth INTEGER NOT NULL
        );
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create directory_roots table: {e}"),
    })
}

fn create_content_cache_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating content_cache table");
    conn.execute_batch(
        "
        CREATE TABLE content_cache (
            document_id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            accessed_at TEXT NOT NULL,
            file_mtime INTEGER NOT NULL
        );
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create content_cache table: {e}"),
    })
}

fn create_views_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating views table");
    conn.execute_batch(
        "
        CREATE TABLE views (
            document_id TEXT PRIMARY KEY,
            view_count INTEGER NOT NULL DEFAULT 0,
            last_viewed TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create views table: {e}"),
    })
}

fn create_fts_table(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating FTS5 full-text search table");
    // Store body directly in FTS5 (not external content mode).
    // The document_id column is unindexed - used only for joining back to
    // documents. Body content is stored only in FTS5 (filesystem is source of
    // truth).
    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE fts_content USING fts5(
            document_id UNINDEXED,
            body,
            tokenize='unicode61'
        );
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create FTS5 table: {e}"),
    })
}

fn create_link_count_triggers(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating link count triggers");
    conn.execute_batch(
        "
        -- Increment link_count on source and backlink_count on target when link inserted
        CREATE TRIGGER trg_links_insert AFTER INSERT ON links
        BEGIN
            UPDATE documents SET link_count = link_count + 1 WHERE id = NEW.source_id;
            UPDATE documents SET backlink_count = backlink_count + 1 WHERE id = NEW.target_id;
        END;

        -- Decrement counts when link deleted
        CREATE TRIGGER trg_links_delete AFTER DELETE ON links
        BEGIN
            UPDATE documents SET link_count = link_count - 1 WHERE id = OLD.source_id;
            UPDATE documents SET backlink_count = backlink_count - 1 WHERE id = OLD.target_id;
        END;
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create link count triggers: {e}"),
    })
}

fn create_view_count_triggers(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating view count triggers");
    conn.execute_batch(
        "
        -- Initialize view_count when view record inserted
        CREATE TRIGGER trg_views_insert AFTER INSERT ON views
        BEGIN
            UPDATE documents SET view_count = NEW.view_count WHERE id = NEW.document_id;
        END;

        -- Update view_count when view record updated
        CREATE TRIGGER trg_views_update AFTER UPDATE OF view_count ON views
        BEGIN
            UPDATE documents SET view_count = NEW.view_count WHERE id = NEW.document_id;
        END;

        -- Reset view_count when view record deleted
        CREATE TRIGGER trg_views_delete AFTER DELETE ON views
        BEGIN
            UPDATE documents SET view_count = 0 WHERE id = OLD.document_id;
        END;
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create view count triggers: {e}"),
    })
}

fn create_fts_triggers(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Creating FTS5 sync triggers");
    // FTS content is managed by fulltext_search module functions.
    // We only need a trigger to clean up FTS when documents are deleted.
    conn.execute_batch(
        "
        -- Clean up FTS index when document deleted
        CREATE TRIGGER trg_fts_delete AFTER DELETE ON documents
        BEGIN
            DELETE FROM fts_content WHERE document_id = OLD.id;
        END;
        ",
    )
    .map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to create FTS triggers: {e}"),
    })
}
