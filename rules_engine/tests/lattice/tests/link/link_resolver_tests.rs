use std::path::Path;

use lattice::id::lattice_id::LatticeId;
use lattice::index::document_queries::insert;
use lattice::index::document_types::InsertDocument;
use lattice::index::schema_definition;
use lattice::link::link_resolver::{LinkResolution, UnresolvedReason, resolve, resolve_batch};
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_test_document(id: &str, path: &str) -> InsertDocument {
    let name = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("test").to_string();
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name,
        "Test document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

fn make_id(s: &str) -> LatticeId {
    s.parse().expect("Valid test ID")
}

#[test]
fn resolve_same_directory() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/features/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.target_id.as_str(), "LTARGT");
            assert_eq!(resolved.relative_path, "system.md");
            assert_eq!(resolved.link_url, "system.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_parent_directory() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "../design/system.md");
            assert_eq!(resolved.link_url, "../design/system.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_nested_subdirectory() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/features/auth/oauth/config.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "auth/oauth/config.md");
            assert_eq!(resolved.link_url, "auth/oauth/config.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_missing_target_returns_unresolved() {
    let conn = create_test_db();

    let source = Path::new("docs/features/auth.md");
    let target_id = make_id("LNOFND");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            panic!("Expected unresolved link, got resolved: {resolved:?}");
        }
        LinkResolution::Unresolved(unresolved) => {
            assert_eq!(unresolved.target_id.as_str(), "LNOFND");
            assert_eq!(unresolved.reason, UnresolvedReason::TargetNotFound);
        }
    }
}

#[test]
fn resolve_deeply_nested_paths() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "project/modules/api/v2/endpoints/users.md"))
        .expect("Insert should succeed");

    let source = Path::new("project/modules/db/migrations/v1/initial.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "../../../api/v2/endpoints/users.md");
            assert_eq!(resolved.link_url, "../../../api/v2/endpoints/users.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_source_at_root() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("readme.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "docs/design/system.md");
            assert_eq!(resolved.link_url, "docs/design/system.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_target_at_root() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "readme.md")).expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "../../readme.md");
            assert_eq!(resolved.link_url, "../../readme.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_both_at_root() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "changelog.md")).expect("Insert should succeed");

    let source = Path::new("readme.md");
    let target_id = make_id("LTARGT");

    let result = resolve(&conn, source, &target_id).expect("Resolve should succeed");

    match result {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "changelog.md");
            assert_eq!(resolved.link_url, "changelog.md#LTARGT");
        }
        LinkResolution::Unresolved(unresolved) => {
            panic!("Expected resolved link, got unresolved: {unresolved:?}");
        }
    }
}

#[test]
fn resolve_batch_multiple_targets() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGA", "docs/a.md")).expect("Insert should succeed");
    insert(&conn, &create_test_document("LTARGB", "docs/b.md")).expect("Insert should succeed");
    insert(&conn, &create_test_document("LTARGC", "api/c.md")).expect("Insert should succeed");

    let source = Path::new("docs/source.md");
    let target_ids = vec![make_id("LTARGA"), make_id("LTARGB"), make_id("LTARGC")];

    let results = resolve_batch(&conn, source, &target_ids).expect("Batch resolve should succeed");

    assert_eq!(results.len(), 3);

    match &results[0] {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "a.md");
        }
        _ => panic!("Expected resolved link"),
    }

    match &results[1] {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "b.md");
        }
        _ => panic!("Expected resolved link"),
    }

    match &results[2] {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.relative_path, "../api/c.md");
        }
        _ => panic!("Expected resolved link"),
    }
}

#[test]
fn resolve_batch_with_missing_targets() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LEXIST", "docs/exists.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/source.md");
    let target_ids = vec![make_id("LEXIST"), make_id("LMISNG")];

    let results = resolve_batch(&conn, source, &target_ids).expect("Batch resolve should succeed");

    assert_eq!(results.len(), 2);

    match &results[0] {
        LinkResolution::Resolved(resolved) => {
            assert_eq!(resolved.target_id.as_str(), "LEXIST");
        }
        _ => panic!("Expected resolved link for LEXIST"),
    }

    match &results[1] {
        LinkResolution::Unresolved(unresolved) => {
            assert_eq!(unresolved.target_id.as_str(), "LMISNG");
            assert_eq!(unresolved.reason, UnresolvedReason::TargetNotFound);
        }
        _ => panic!("Expected unresolved link for LMISNG"),
    }
}

#[test]
fn resolve_batch_empty() {
    let conn = create_test_db();

    let source = Path::new("docs/source.md");
    let target_ids: Vec<LatticeId> = vec![];

    let results = resolve_batch(&conn, source, &target_ids).expect("Empty batch should succeed");

    assert!(results.is_empty());
}
