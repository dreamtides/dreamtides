use lattice::index::schema_definition::{
    CONTENT_CACHE_MAX_ENTRIES, SCHEMA_VERSION, create_schema, optimize_fts, schema_is_current,
    schema_version,
};
use rusqlite::Connection;

fn create_in_memory_db() -> Connection {
    Connection::open_in_memory().expect("Failed to create in-memory database")
}

#[test]
fn schema_version_constant_is_positive() {
    assert!(SCHEMA_VERSION > 0, "Schema version should be positive");
}

#[test]
fn content_cache_max_entries_constant_is_reasonable() {
    assert!(
        CONTENT_CACHE_MAX_ENTRIES >= 10 && CONTENT_CACHE_MAX_ENTRIES <= 10000,
        "Content cache max entries should be between 10 and 10000"
    );
}

#[test]
fn schema_version_returns_none_for_empty_database() {
    let conn = create_in_memory_db();

    let result = schema_version(&conn).expect("Should not error on empty database");

    assert!(result.is_none(), "Empty database should have no schema version");
}

#[test]
fn schema_is_current_returns_false_for_empty_database() {
    let conn = create_in_memory_db();

    let result = schema_is_current(&conn).expect("Should not error on empty database");

    assert!(!result, "Empty database should not be current");
}

#[test]
fn create_schema_succeeds_on_empty_database() {
    let conn = create_in_memory_db();

    let result = create_schema(&conn);

    assert!(result.is_ok(), "Schema creation should succeed: {result:?}");
}

#[test]
fn schema_version_returns_current_after_creation() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let version = schema_version(&conn)
        .expect("Should read schema version")
        .expect("Schema version should exist after creation");

    assert_eq!(version, SCHEMA_VERSION, "Schema version should match constant");
}

#[test]
fn schema_is_current_returns_true_after_creation() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let result = schema_is_current(&conn).expect("Should check schema version");

    assert!(result, "Schema should be current after creation");
}

#[test]
fn documents_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(documents)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    let expected = vec![
        "id",
        "parent_id",
        "path",
        "name",
        "description",
        "task_type",
        "is_closed",
        "priority",
        "created_at",
        "updated_at",
        "closed_at",
        "body_hash",
        "indexed_at",
        "content_length",
        "link_count",
        "backlink_count",
        "view_count",
        "is_root",
        "in_tasks_dir",
        "in_docs_dir",
    ];
    for col in &expected {
        assert!(columns.contains(&col.to_string()), "documents table should have column: {col}");
    }
}

#[test]
fn links_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(links)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(columns.contains(&"source_id".to_string()), "links should have source_id");
    assert!(columns.contains(&"target_id".to_string()), "links should have target_id");
    assert!(columns.contains(&"link_type".to_string()), "links should have link_type");
    assert!(columns.contains(&"position".to_string()), "links should have position");
}

#[test]
fn labels_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(labels)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(columns.contains(&"document_id".to_string()), "labels should have document_id");
    assert!(columns.contains(&"label".to_string()), "labels should have label");
}

#[test]
fn views_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(views)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(columns.contains(&"document_id".to_string()), "views should have document_id");
    assert!(columns.contains(&"view_count".to_string()), "views should have view_count");
    assert!(columns.contains(&"last_viewed".to_string()), "views should have last_viewed");
}

#[test]
fn content_cache_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(content_cache)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(columns.contains(&"document_id".to_string()), "content_cache should have document_id");
    assert!(columns.contains(&"content".to_string()), "content_cache should have content");
    assert!(
        columns.contains(&"content_hash".to_string()),
        "content_cache should have content_hash"
    );
    assert!(columns.contains(&"accessed_at".to_string()), "content_cache should have accessed_at");
    assert!(columns.contains(&"file_mtime".to_string()), "content_cache should have file_mtime");
}

#[test]
fn client_counters_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(client_counters)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(columns.contains(&"client_id".to_string()), "client_counters should have client_id");
    assert!(
        columns.contains(&"next_counter".to_string()),
        "client_counters should have next_counter"
    );
}

#[test]
fn directory_roots_table_has_expected_columns() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(directory_roots)")
        .expect("Should prepare pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("Should query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("Should collect column names");

    assert!(
        columns.contains(&"directory_path".to_string()),
        "directory_roots should have directory_path"
    );
    assert!(columns.contains(&"root_id".to_string()), "directory_roots should have root_id");
    assert!(
        columns.contains(&"parent_path".to_string()),
        "directory_roots should have parent_path"
    );
    assert!(columns.contains(&"depth".to_string()), "directory_roots should have depth");
}

#[test]
fn link_insert_trigger_increments_counts() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LSRC1', 'src.md', 'src', 'Source', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert source document");
    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LTGT1', 'tgt.md', 'tgt', 'Target', 'def', datetime('now'), 100)",
        [],
    )
    .expect("Should insert target document");

    conn.execute("INSERT INTO links (source_id, target_id, link_type, position) VALUES ('LSRC1', 'LTGT1', 'body', 0)", [])
        .expect("Should insert link");

    let source_link_count: i32 = conn
        .query_row("SELECT link_count FROM documents WHERE id = 'LSRC1'", [], |row| row.get(0))
        .unwrap();
    let target_backlink_count: i32 = conn
        .query_row("SELECT backlink_count FROM documents WHERE id = 'LTGT1'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(source_link_count, 1, "Source document link_count should be 1");
    assert_eq!(target_backlink_count, 1, "Target document backlink_count should be 1");
}

#[test]
fn link_delete_trigger_decrements_counts() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LSRC2', 'src.md', 'src', 'Source', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert source document");
    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LTGT2', 'tgt.md', 'tgt', 'Target', 'def', datetime('now'), 100)",
        [],
    )
    .expect("Should insert target document");
    conn.execute("INSERT INTO links (source_id, target_id, link_type, position) VALUES ('LSRC2', 'LTGT2', 'body', 0)", [])
        .expect("Should insert link");

    conn.execute("DELETE FROM links WHERE source_id = 'LSRC2' AND target_id = 'LTGT2'", [])
        .expect("Should delete link");

    let source_link_count: i32 = conn
        .query_row("SELECT link_count FROM documents WHERE id = 'LSRC2'", [], |row| row.get(0))
        .unwrap();
    let target_backlink_count: i32 = conn
        .query_row("SELECT backlink_count FROM documents WHERE id = 'LTGT2'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(source_link_count, 0, "Source document link_count should be 0 after delete");
    assert_eq!(target_backlink_count, 0, "Target document backlink_count should be 0 after delete");
}

#[test]
fn view_insert_trigger_updates_document_view_count() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LDOC1', 'doc.md', 'doc', 'Document', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert document");

    conn.execute("INSERT INTO views (document_id, view_count, last_viewed) VALUES ('LDOC1', 5, datetime('now'))", [])
        .expect("Should insert view record");

    let doc_view_count: i32 = conn
        .query_row("SELECT view_count FROM documents WHERE id = 'LDOC1'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(doc_view_count, 5, "Document view_count should be synced from views table");
}

#[test]
fn view_update_trigger_updates_document_view_count() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LDOC2', 'doc.md', 'doc', 'Document', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert document");
    conn.execute("INSERT INTO views (document_id, view_count, last_viewed) VALUES ('LDOC2', 5, datetime('now'))", [])
        .expect("Should insert view record");

    conn.execute("UPDATE views SET view_count = 10 WHERE document_id = 'LDOC2'", [])
        .expect("Should update view record");

    let doc_view_count: i32 = conn
        .query_row("SELECT view_count FROM documents WHERE id = 'LDOC2'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(doc_view_count, 10, "Document view_count should be updated when views updated");
}

#[test]
fn view_delete_trigger_resets_document_view_count() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LDOC3', 'doc.md', 'doc', 'Document', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert document");
    conn.execute("INSERT INTO views (document_id, view_count, last_viewed) VALUES ('LDOC3', 5, datetime('now'))", [])
        .expect("Should insert view record");

    conn.execute("DELETE FROM views WHERE document_id = 'LDOC3'", [])
        .expect("Should delete view record");

    let doc_view_count: i32 = conn
        .query_row("SELECT view_count FROM documents WHERE id = 'LDOC3'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(doc_view_count, 0, "Document view_count should be reset to 0 when views deleted");
}

#[test]
fn fts_table_exists_after_schema_creation() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='fts_content'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    assert!(exists, "FTS5 table fts_content should exist");
}

#[test]
fn optimize_fts_succeeds_after_schema_creation() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let result = optimize_fts(&conn);

    assert!(result.is_ok(), "FTS optimization should succeed: {result:?}");
}

#[test]
fn index_metadata_has_single_row_constraint() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    let result = conn.execute(
        "INSERT INTO index_metadata (id, schema_version, last_indexed) VALUES (2, 1, datetime('now'))",
        [],
    );

    assert!(result.is_err(), "Should not allow inserting row with id != 1");
}

#[test]
fn documents_path_has_unique_constraint() {
    let conn = create_in_memory_db();
    create_schema(&conn).expect("Schema creation should succeed");

    conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LDOC1', 'same/path.md', 'doc1', 'Doc 1', 'abc', datetime('now'), 100)",
        [],
    )
    .expect("Should insert first document");

    let result = conn.execute(
        "INSERT INTO documents (id, path, name, description, body_hash, indexed_at, content_length)
         VALUES ('LDOC2', 'same/path.md', 'doc2', 'Doc 2', 'def', datetime('now'), 100)",
        [],
    );

    assert!(result.is_err(), "Should reject duplicate path");
}
