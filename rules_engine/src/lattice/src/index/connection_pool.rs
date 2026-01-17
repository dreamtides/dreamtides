use std::path::{Path, PathBuf};
use std::time::Instant;

use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;

/// Default mmap size in bytes (256MB).
const DEFAULT_MMAP_SIZE: i64 = 268_435_456;
/// Busy timeout in milliseconds.
const BUSY_TIMEOUT_MS: i32 = 5000;
/// Relative path from the repository root to the index database.
pub const INDEX_PATH: &str = ".lattice/index.sqlite";

/// Opens a new SQLite connection to the index database with optimized settings.
///
/// The connection is configured with:
/// - WAL journal mode for concurrent readers/writers
/// - NORMAL synchronous mode (safe with WAL, acceptable for rebuildable cache)
/// - Memory-based temp store
/// - Memory-mapped I/O (256MB by default, disabled on network filesystems)
/// - 5 second busy timeout for concurrent access
pub fn open_connection(repo_root: &Path) -> Result<Connection, LatticeError> {
    let index_path = repo_root.join(INDEX_PATH);
    open_connection_at(&index_path)
}

/// Opens a connection to a specific database path.
///
/// This is the core connection function that configures all SQLite PRAGMAs.
/// Use [`open_connection`] for normal index access.
pub fn open_connection_at(db_path: &Path) -> Result<Connection, LatticeError> {
    let start = Instant::now();
    debug!(?db_path, "Opening SQLite connection");

    let conn = Connection::open(db_path).map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to open database at {}: {e}", db_path.display()),
    })?;

    configure_connection(&conn, db_path)?;

    let elapsed = start.elapsed();
    info!(elapsed_ms = elapsed.as_millis(), "Connection opened and configured");

    Ok(conn)
}

/// Creates a new in-memory SQLite connection with standard configuration.
///
/// Useful for testing or temporary operations that don't need persistence.
pub fn open_memory_connection() -> Result<Connection, LatticeError> {
    let start = Instant::now();
    debug!("Opening in-memory SQLite connection");

    let conn = Connection::open_in_memory().map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to open in-memory database: {e}"),
    })?;

    configure_memory_connection(&conn)?;

    let elapsed = start.elapsed();
    debug!(elapsed_ms = elapsed.as_millis(), "In-memory connection opened and configured");

    Ok(conn)
}

/// Runs `PRAGMA optimize` to analyze and optimize the database.
///
/// Call this before closing a connection that performed significant writes.
pub fn optimize(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Running PRAGMA optimize");
    conn.execute_batch("PRAGMA optimize;").map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to run PRAGMA optimize: {e}"),
    })
}

/// Runs a WAL checkpoint to truncate the write-ahead log.
///
/// Call this after bulk operations (like a full index rebuild) to reclaim
/// disk space and ensure durability. The TRUNCATE mode resets the WAL file
/// to zero bytes after checkpointing all frames.
pub fn checkpoint(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Running WAL checkpoint (TRUNCATE)");
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to run WAL checkpoint: {e}") }
    })
}

/// Returns the path to the index database for a given repository root.
pub fn index_path(repo_root: &Path) -> PathBuf {
    repo_root.join(INDEX_PATH)
}

/// Ensures the `.lattice` directory exists for the index database.
pub fn ensure_lattice_dir(repo_root: &Path) -> Result<(), LatticeError> {
    let lattice_dir = repo_root.join(".lattice");
    if !lattice_dir.exists() {
        debug!(?lattice_dir, "Creating .lattice directory");
        std::fs::create_dir_all(&lattice_dir).map_err(|e| LatticeError::WriteError {
            path: lattice_dir,
            reason: format!("Failed to create .lattice directory: {e}"),
        })?;
    }
    Ok(())
}

/// Deletes the index database and associated WAL/SHM files.
///
/// This is safe to call even if the files don't exist.
pub fn delete_index(repo_root: &Path) -> Result<(), LatticeError> {
    let base_path = index_path(repo_root);
    delete_database_files(&base_path)
}

fn configure_connection(conn: &Connection, db_path: &Path) -> Result<(), LatticeError> {
    set_journal_mode_wal(conn)?;
    set_synchronous_normal(conn)?;
    set_temp_store_memory(conn)?;
    set_mmap_size(conn, db_path)?;
    set_busy_timeout(conn)?;
    Ok(())
}

fn configure_memory_connection(conn: &Connection) -> Result<(), LatticeError> {
    set_temp_store_memory(conn)?;
    Ok(())
}

fn set_journal_mode_wal(conn: &Connection) -> Result<(), LatticeError> {
    let result: String =
        conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError { reason: format!("Failed to set journal_mode: {e}") }
        })?;

    if result.to_lowercase() != "wal" {
        warn!(actual = result, "journal_mode is not WAL (may be on network filesystem)");
    } else {
        debug!("journal_mode = WAL");
    }

    Ok(())
}

fn set_synchronous_normal(conn: &Connection) -> Result<(), LatticeError> {
    conn.execute_batch("PRAGMA synchronous = NORMAL;").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to set synchronous mode: {e}") }
    })?;
    debug!("synchronous = NORMAL");
    Ok(())
}

fn set_temp_store_memory(conn: &Connection) -> Result<(), LatticeError> {
    conn.execute_batch("PRAGMA temp_store = MEMORY;").map_err(|e| LatticeError::DatabaseError {
        reason: format!("Failed to set temp_store: {e}"),
    })?;
    debug!("temp_store = MEMORY");
    Ok(())
}

fn set_mmap_size(conn: &Connection, db_path: &Path) -> Result<(), LatticeError> {
    let mmap_size = if is_likely_network_filesystem(db_path) {
        warn!(?db_path, "Database appears to be on network filesystem, disabling mmap");
        0
    } else {
        DEFAULT_MMAP_SIZE
    };

    conn.execute_batch(&format!("PRAGMA mmap_size = {mmap_size};")).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to set mmap_size: {e}") }
    })?;
    debug!(mmap_size, "mmap_size configured");
    Ok(())
}

fn set_busy_timeout(conn: &Connection) -> Result<(), LatticeError> {
    conn.execute_batch(&format!("PRAGMA busy_timeout = {BUSY_TIMEOUT_MS};")).map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to set busy_timeout: {e}") }
    })?;
    debug!(timeout_ms = BUSY_TIMEOUT_MS, "busy_timeout configured");
    Ok(())
}

fn is_likely_network_filesystem(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let network_prefixes = [
        "/Volumes/",
        "/mnt/",
        "/run/user/",
        "/media/",
        "//",
        "/net/",
        "/afs/",
        "/cifs/",
        "/smb/",
        "/nfs/",
    ];

    network_prefixes.iter().any(|prefix| path_str.contains(prefix))
}

fn delete_database_files(base_path: &Path) -> Result<(), LatticeError> {
    let files_to_delete = [
        base_path.to_path_buf(),
        base_path.with_extension("sqlite-wal"),
        base_path.with_extension("sqlite-shm"),
    ];

    for file in &files_to_delete {
        if file.exists() {
            debug!(?file, "Deleting database file");
            std::fs::remove_file(file).map_err(|e| LatticeError::WriteError {
                path: file.clone(),
                reason: format!("Failed to delete database file: {e}"),
            })?;
        }
    }

    info!(?base_path, "Database files deleted");
    Ok(())
}
