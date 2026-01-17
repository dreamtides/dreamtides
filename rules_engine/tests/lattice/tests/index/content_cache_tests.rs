use std::io::Write;
use std::time::Duration;
use std::{fs, thread};

use lattice::index::content_cache::{
    clear_cache, get_cache_stats, get_cached_content, invalidate_cache, put_content,
};
use lattice::index::schema_definition;
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create test file");
    file.write_all(content.as_bytes()).expect("Failed to write test file");
    path
}

fn get_file_mtime(path: &std::path::Path) -> u64 {
    use std::time::SystemTime;
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

#[test]
fn put_and_get_cached_content() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "# Test Document\n\nSome content.");
    let mtime = get_file_mtime(&path);

    put_content(&conn, "LDOC1", "# Test Document\n\nSome content.", mtime)
        .expect("Put content should succeed");

    let cached = get_cached_content(&conn, "LDOC1", &path)
        .expect("Get should succeed")
        .expect("Should have cached content");

    assert_eq!(cached.document_id, "LDOC1");
    assert_eq!(cached.content, "# Test Document\n\nSome content.");
    assert!(!cached.content_hash.is_empty(), "Hash should be computed");
}

#[test]
fn get_returns_none_for_uncached_document() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "content");

    let cached =
        get_cached_content(&conn, "LDOC2", &path).expect("Get should succeed even for miss");

    assert!(cached.is_none(), "Uncached document should return None");
}

#[test]
fn get_returns_none_when_file_modified() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "original content");
    let mtime = get_file_mtime(&path);

    let old_mtime = mtime.saturating_sub(10);
    put_content(&conn, "LDOC3", "original content", old_mtime).expect("Put should succeed");

    let cached = get_cached_content(&conn, "LDOC3", &path).expect("Get should succeed");

    assert!(cached.is_none(), "Stale cache entry should return None when mtime differs");
}

#[test]
fn put_replaces_existing_entry() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "new content");
    let mtime = get_file_mtime(&path);

    put_content(&conn, "LDOC4", "old content", mtime).expect("First put should succeed");
    put_content(&conn, "LDOC4", "new content", mtime).expect("Second put should succeed");

    let cached = get_cached_content(&conn, "LDOC4", &path)
        .expect("Get should succeed")
        .expect("Should have cached content");

    assert_eq!(cached.content, "new content", "Content should be updated");
}

#[test]
fn invalidate_removes_entry() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "content");
    let mtime = get_file_mtime(&path);

    put_content(&conn, "LDOC5", "content", mtime).expect("Put should succeed");

    let removed = invalidate_cache(&conn, "LDOC5").expect("Invalidate should succeed");

    assert!(removed, "Should return true when entry was removed");

    let cached = get_cached_content(&conn, "LDOC5", &path).expect("Get should succeed");
    assert!(cached.is_none(), "Cache should be empty after invalidation");
}

#[test]
fn invalidate_returns_false_for_nonexistent() {
    let conn = create_test_db();

    let removed = invalidate_cache(&conn, "LDOC6").expect("Invalidate should succeed");

    assert!(!removed, "Should return false when no entry existed");
}

#[test]
fn clear_removes_all_entries() {
    let conn = create_test_db();

    put_content(&conn, "LDOC7A", "content a", 1000).expect("Put should succeed");
    put_content(&conn, "LDOC7B", "content b", 1001).expect("Put should succeed");
    put_content(&conn, "LDOC7C", "content c", 1002).expect("Put should succeed");

    let cleared = clear_cache(&conn).expect("Clear should succeed");

    assert_eq!(cleared, 3, "Should clear all 3 entries");

    let stats = get_cache_stats(&conn).expect("Stats should succeed");
    assert_eq!(stats.entry_count, 0, "Cache should be empty");
}

#[test]
fn get_cache_stats_returns_correct_counts() {
    let conn = create_test_db();

    let stats = get_cache_stats(&conn).expect("Stats should succeed");
    assert_eq!(stats.entry_count, 0, "Empty cache should have 0 entries");
    assert_eq!(stats.total_content_bytes, 0, "Empty cache should have 0 bytes");

    put_content(&conn, "LDOC8A", "hello", 1000).expect("Put should succeed");
    put_content(&conn, "LDOC8B", "world!", 1001).expect("Put should succeed");

    let stats = get_cache_stats(&conn).expect("Stats should succeed");
    assert_eq!(stats.entry_count, 2, "Should have 2 entries");
    assert_eq!(stats.total_content_bytes, 11, "Should have 11 bytes (5 + 6)");
}

#[test]
fn eviction_removes_oldest_entries() {
    let conn = create_test_db();

    for i in 0..105 {
        let id = format!("LDOC{i:03}");
        let content = format!("content {i}");
        put_content(&conn, &id, &content, 1000 + i).expect("Put should succeed");

        thread::sleep(Duration::from_millis(2));
    }

    let stats = get_cache_stats(&conn).expect("Stats should succeed");
    assert_eq!(stats.entry_count, 100, "Cache should be capped at 100 entries");

    let first_entry: Result<String, _> = conn.query_row(
        "SELECT document_id FROM content_cache WHERE document_id = ?",
        ["LDOC000"],
        |row| row.get(0),
    );
    assert!(first_entry.is_err(), "First entry should have been evicted");

    let last_entry: String = conn
        .query_row(
            "SELECT document_id FROM content_cache WHERE document_id = ?",
            ["LDOC104"],
            |row| row.get(0),
        )
        .expect("Last entry should exist");
    assert_eq!(last_entry, "LDOC104", "Most recent entry should still exist");
}

#[test]
fn get_updates_accessed_at_on_cache_hit() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "test content");
    let mtime = get_file_mtime(&path);

    put_content(&conn, "LDOC10", "test content", mtime).expect("Put should succeed");

    let first_accessed: String = conn
        .query_row(
            "SELECT accessed_at FROM content_cache WHERE document_id = ?",
            ["LDOC10"],
            |row| row.get(0),
        )
        .expect("Query should succeed");

    thread::sleep(Duration::from_millis(10));
    get_cached_content(&conn, "LDOC10", &path).expect("Get should succeed");

    let second_accessed: String = conn
        .query_row(
            "SELECT accessed_at FROM content_cache WHERE document_id = ?",
            ["LDOC10"],
            |row| row.get(0),
        )
        .expect("Query should succeed");

    assert!(second_accessed >= first_accessed, "accessed_at should be updated on cache hit");
}

#[test]
fn content_hash_is_deterministic() {
    let conn = create_test_db();
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = create_test_file(&dir, "doc.md", "same content");
    let mtime = get_file_mtime(&path);

    put_content(&conn, "LDOC11A", "same content", mtime).expect("Put should succeed");
    put_content(&conn, "LDOC11B", "same content", mtime).expect("Put should succeed");

    let hash_a: String = conn
        .query_row(
            "SELECT content_hash FROM content_cache WHERE document_id = ?",
            ["LDOC11A"],
            |row| row.get(0),
        )
        .expect("Query should succeed");
    let hash_b: String = conn
        .query_row(
            "SELECT content_hash FROM content_cache WHERE document_id = ?",
            ["LDOC11B"],
            |row| row.get(0),
        )
        .expect("Query should succeed");

    assert_eq!(hash_a, hash_b, "Same content should produce same hash");
}

#[test]
fn content_hash_differs_for_different_content() {
    let conn = create_test_db();

    put_content(&conn, "LDOC12A", "content one", 1000).expect("Put should succeed");
    put_content(&conn, "LDOC12B", "content two", 1001).expect("Put should succeed");

    let hash_a: String = conn
        .query_row(
            "SELECT content_hash FROM content_cache WHERE document_id = ?",
            ["LDOC12A"],
            |row| row.get(0),
        )
        .expect("Query should succeed");
    let hash_b: String = conn
        .query_row(
            "SELECT content_hash FROM content_cache WHERE document_id = ?",
            ["LDOC12B"],
            |row| row.get(0),
        )
        .expect("Query should succeed");

    assert_ne!(hash_a, hash_b, "Different content should produce different hashes");
}

#[test]
fn clear_returns_zero_when_empty() {
    let conn = create_test_db();

    let cleared = clear_cache(&conn).expect("Clear should succeed");

    assert_eq!(cleared, 0, "Should return 0 when cache is empty");
}

#[test]
fn get_returns_none_for_nonexistent_file() {
    let conn = create_test_db();
    let nonexistent_path = std::path::Path::new("/nonexistent/path/doc.md");

    put_content(&conn, "LDOC13", "content", 1234).expect("Put should succeed");

    let cached = get_cached_content(&conn, "LDOC13", nonexistent_path).expect("Get should succeed");

    assert!(cached.is_none(), "Should return None when file doesn't exist (mtime would be 0)");
}
