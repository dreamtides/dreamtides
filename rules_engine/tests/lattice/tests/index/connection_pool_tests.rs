use std::path::Path;

use lattice::index::connection_pool;
use tempfile::TempDir;

#[test]
fn open_connection_creates_database_file() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    assert!(
        connection_pool::index_path(temp_dir.path()).exists(),
        "index file should exist after opening connection"
    );

    drop(conn);
}

#[test]
fn open_connection_configures_wal_mode() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .expect("should query journal_mode");

    assert_eq!(journal_mode.to_lowercase(), "wal", "journal_mode should be WAL");
}

#[test]
fn open_connection_configures_synchronous_normal() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    let synchronous: i32 = conn
        .query_row("PRAGMA synchronous;", [], |row| row.get(0))
        .expect("should query synchronous mode");

    assert_eq!(synchronous, 1, "synchronous mode should be NORMAL (1)");
}

#[test]
fn open_connection_configures_temp_store_memory() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    let temp_store: i32 = conn
        .query_row("PRAGMA temp_store;", [], |row| row.get(0))
        .expect("should query temp_store");

    assert_eq!(temp_store, 2, "temp_store should be MEMORY (2)");
}

#[test]
fn open_connection_configures_busy_timeout() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    let busy_timeout: i32 = conn
        .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
        .expect("should query busy_timeout");

    assert_eq!(busy_timeout, 5000, "busy_timeout should be 5000ms");
}

#[test]
fn open_memory_connection_works() {
    let conn = connection_pool::open_memory_connection().expect("should open in-memory connection");

    conn.execute_batch("CREATE TABLE test (id INTEGER);")
        .expect("should be able to create table in memory connection");
}

#[test]
fn optimize_runs_without_error() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    conn.execute_batch("CREATE TABLE test (id INTEGER);").expect("should create table");

    connection_pool::optimize(&conn).expect("optimize should succeed");
}

#[test]
fn checkpoint_runs_without_error() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    conn.execute_batch("CREATE TABLE test (id INTEGER); INSERT INTO test VALUES (1);")
        .expect("should create table and insert data");

    connection_pool::checkpoint(&conn).expect("checkpoint should succeed");
}

#[test]
fn ensure_lattice_dir_creates_directory() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");

    assert!(!lattice_dir.exists(), ".lattice should not exist initially");

    connection_pool::ensure_lattice_dir(temp_dir.path())
        .expect("ensure_lattice_dir should succeed");

    assert!(lattice_dir.exists(), ".lattice should exist after ensure_lattice_dir");
}

#[test]
fn ensure_lattice_dir_is_idempotent() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    connection_pool::ensure_lattice_dir(temp_dir.path())
        .expect("first ensure_lattice_dir should succeed");
    connection_pool::ensure_lattice_dir(temp_dir.path())
        .expect("second ensure_lattice_dir should also succeed");
}

#[test]
fn delete_index_removes_database_files() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    conn.execute_batch("CREATE TABLE test (id INTEGER);").expect("should create table");

    drop(conn);

    let index_path = connection_pool::index_path(temp_dir.path());
    assert!(index_path.exists(), "index should exist before delete");

    connection_pool::delete_index(temp_dir.path()).expect("delete_index should succeed");

    assert!(!index_path.exists(), "index should not exist after delete");
}

#[test]
fn delete_index_succeeds_when_files_do_not_exist() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    connection_pool::delete_index(temp_dir.path())
        .expect("delete_index should succeed even when no files exist");
}

#[test]
fn index_path_returns_correct_path() {
    let repo_root = Path::new("/fake/repo");
    let expected = repo_root.join(".lattice/index.sqlite");

    assert_eq!(connection_pool::index_path(repo_root), expected, "index_path should be correct");
}

#[test]
fn index_path_constant_matches_expected_value() {
    assert_eq!(
        connection_pool::INDEX_PATH,
        ".lattice/index.sqlite",
        "INDEX_PATH constant should have correct value"
    );
}

#[test]
fn open_connection_configures_mmap_size() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");

    let mmap_size: i64 =
        conn.query_row("PRAGMA mmap_size;", [], |row| row.get(0)).expect("should query mmap_size");

    assert_eq!(mmap_size, 268_435_456, "mmap_size should be 256MB (268435456 bytes)");
}
