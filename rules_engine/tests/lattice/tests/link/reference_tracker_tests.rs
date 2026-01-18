use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType, insert_for_document};
use lattice::index::{document_queries, schema_definition};
use lattice::link::reference_tracker::{
    find_orphans, query_forward, query_forward_by_type, query_reverse, query_reverse_by_type,
};
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_test_document(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Description for {name}"),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

fn setup_documents(conn: &Connection, ids: &[&str]) {
    for id in ids {
        let doc = create_test_document(id, &format!("docs/{id}.md"), id);
        document_queries::insert(conn, &doc).expect("Failed to insert document");
    }
}

fn insert_links(conn: &Connection, links: Vec<(&str, &str, LinkType, u32)>) {
    let link_data: Vec<InsertLink> = links
        .iter()
        .map(|(src, tgt, lt, pos)| InsertLink {
            source_id: src,
            target_id: tgt,
            link_type: *lt,
            position: *pos,
        })
        .collect();
    insert_for_document(conn, &link_data).expect("Failed to insert links");
}

#[test]
fn query_forward_returns_all_outgoing_references() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCA", "LTGTA", "LTGTB"]);
    insert_links(&conn, vec![
        ("LSRCA", "LTGTA", LinkType::Body, 0),
        ("LSRCA", "LTGTB", LinkType::Body, 1),
    ]);

    let result = query_forward(&conn, "LSRCA").expect("Query should succeed");

    assert_eq!(result.source_id, "LSRCA");
    assert_eq!(result.references.len(), 2, "Should have 2 forward references");
    assert_eq!(result.references[0].document.id, "LTGTA");
    assert_eq!(result.references[1].document.id, "LTGTB");
}

#[test]
fn query_forward_returns_empty_for_document_with_no_links() {
    let conn = create_test_db();
    setup_documents(&conn, &["LISOLN"]);

    let result = query_forward(&conn, "LISOLN").expect("Query should succeed");

    assert_eq!(result.source_id, "LISOLN");
    assert!(result.references.is_empty(), "Should have no forward references");
}

#[test]
fn query_forward_preserves_position_order() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCB", "LTGTC", "LTGTD", "LTGTE"]);
    insert_links(&conn, vec![
        ("LSRCB", "LTGTE", LinkType::Body, 2),
        ("LSRCB", "LTGTC", LinkType::Body, 0),
        ("LSRCB", "LTGTD", LinkType::Body, 1),
    ]);

    let result = query_forward(&conn, "LSRCB").expect("Query should succeed");

    assert_eq!(result.references[0].document.id, "LTGTC", "Position 0 should be first");
    assert_eq!(result.references[1].document.id, "LTGTD", "Position 1 should be second");
    assert_eq!(result.references[2].document.id, "LTGTE", "Position 2 should be third");
}

#[test]
fn query_forward_by_type_filters_correctly() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCC", "LTGTF", "LTGTG", "LTGTH"]);
    insert_links(&conn, vec![
        ("LSRCC", "LTGTF", LinkType::Body, 0),
        ("LSRCC", "LTGTG", LinkType::BlockedBy, 1),
        ("LSRCC", "LTGTH", LinkType::Blocking, 2),
    ]);

    let body_refs =
        query_forward_by_type(&conn, "LSRCC", LinkType::Body).expect("Query should succeed");
    let blocked_by_refs =
        query_forward_by_type(&conn, "LSRCC", LinkType::BlockedBy).expect("Query should succeed");

    assert_eq!(body_refs.references.len(), 1, "Should have 1 body reference");
    assert_eq!(body_refs.references[0].document.id, "LTGTF");
    assert_eq!(blocked_by_refs.references.len(), 1, "Should have 1 blocked_by reference");
    assert_eq!(blocked_by_refs.references[0].document.id, "LTGTG");
}

#[test]
fn query_reverse_returns_all_backlinks() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCD", "LSRCE", "LTGTI"]);
    insert_links(&conn, vec![
        ("LSRCD", "LTGTI", LinkType::Body, 0),
        ("LSRCE", "LTGTI", LinkType::Body, 0),
    ]);

    let result = query_reverse(&conn, "LTGTI").expect("Query should succeed");

    assert_eq!(result.target_id, "LTGTI");
    assert_eq!(result.references.len(), 2, "Should have 2 backlinks");
    let source_ids: Vec<&str> = result.references.iter().map(|r| r.document.id.as_str()).collect();
    assert!(source_ids.contains(&"LSRCD"), "Should include LSRCD");
    assert!(source_ids.contains(&"LSRCE"), "Should include LSRCE");
}

#[test]
fn query_reverse_returns_empty_for_orphan() {
    let conn = create_test_db();
    setup_documents(&conn, &["LORPNA"]);

    let result = query_reverse(&conn, "LORPNA").expect("Query should succeed");

    assert_eq!(result.target_id, "LORPNA");
    assert!(result.references.is_empty(), "Orphan should have no backlinks");
}

#[test]
fn query_reverse_by_type_filters_correctly() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCF", "LSRCG", "LTGTJ"]);
    insert_links(&conn, vec![
        ("LSRCF", "LTGTJ", LinkType::Body, 0),
        ("LSRCG", "LTGTJ", LinkType::BlockedBy, 0),
    ]);

    let body_backlinks =
        query_reverse_by_type(&conn, "LTGTJ", LinkType::Body).expect("Query should succeed");
    let blocked_by_backlinks =
        query_reverse_by_type(&conn, "LTGTJ", LinkType::BlockedBy).expect("Query should succeed");

    assert_eq!(body_backlinks.references.len(), 1, "Should have 1 body backlink");
    assert_eq!(body_backlinks.references[0].document.id, "LSRCF");
    assert_eq!(blocked_by_backlinks.references.len(), 1, "Should have 1 blocked_by backlink");
    assert_eq!(blocked_by_backlinks.references[0].document.id, "LSRCG");
}

#[test]
fn find_orphans_returns_documents_with_no_backlinks() {
    let conn = create_test_db();
    setup_documents(&conn, &["LORPNB", "LTGTK", "LTGTL"]);
    insert_links(&conn, vec![
        ("LORPNB", "LTGTK", LinkType::Body, 0),
        ("LTGTK", "LTGTL", LinkType::Body, 0),
    ]);

    let orphans = find_orphans(&conn).expect("Find orphans should succeed");

    let orphan_ids: Vec<&str> = orphans.iter().map(|d| d.id.as_str()).collect();
    assert!(orphan_ids.contains(&"LORPNB"), "LORPNB should be orphan (no incoming links)");
    assert!(!orphan_ids.contains(&"LTGTK"), "LTGTK should not be orphan (has incoming)");
    assert!(!orphan_ids.contains(&"LTGTL"), "LTGTL should not be orphan (has incoming)");
}

#[test]
fn find_orphans_returns_all_documents_when_no_links() {
    let conn = create_test_db();
    setup_documents(&conn, &["LORPNC", "LORPND"]);

    let orphans = find_orphans(&conn).expect("Find orphans should succeed");

    assert_eq!(orphans.len(), 2, "All documents should be orphans");
}

#[test]
fn reference_includes_link_type() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCH", "LTGTM"]);
    insert_links(&conn, vec![("LSRCH", "LTGTM", LinkType::DiscoveredFrom, 0)]);

    let result = query_forward(&conn, "LSRCH").expect("Query should succeed");

    assert_eq!(result.references[0].link_type, LinkType::DiscoveredFrom);
}

#[test]
fn reference_includes_position() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCI", "LTGTN"]);
    insert_links(&conn, vec![("LSRCI", "LTGTN", LinkType::Body, 42)]);

    let result = query_forward(&conn, "LSRCI").expect("Query should succeed");

    assert_eq!(result.references[0].position, 42);
}

#[test]
fn reference_includes_full_document_metadata() {
    let conn = create_test_db();
    let doc = InsertDocument::new(
        "LTGTO".to_string(),
        None,
        "docs/target.md".to_string(),
        "target".to_string(),
        "Target document description".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        500,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");
    setup_documents(&conn, &["LSRCJ"]);
    insert_links(&conn, vec![("LSRCJ", "LTGTO", LinkType::Body, 0)]);

    let result = query_forward(&conn, "LSRCJ").expect("Query should succeed");

    assert_eq!(result.references[0].document.id, "LTGTO");
    assert_eq!(result.references[0].document.path, "docs/target.md");
    assert_eq!(result.references[0].document.name, "target");
    assert_eq!(result.references[0].document.description, "Target document description");
}

#[test]
fn query_forward_skips_missing_target_documents() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCK", "LTGTP"]);
    insert_links(&conn, vec![
        ("LSRCK", "LTGTP", LinkType::Body, 0),
        ("LSRCK", "LMISNG", LinkType::Body, 1),
    ]);

    let result = query_forward(&conn, "LSRCK").expect("Query should succeed");

    assert_eq!(result.references.len(), 1, "Should skip missing document");
    assert_eq!(result.references[0].document.id, "LTGTP");
}

#[test]
fn query_reverse_skips_missing_source_documents() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCL", "LTGTQ"]);
    insert_links(&conn, vec![
        ("LSRCL", "LTGTQ", LinkType::Body, 0),
        ("LMISNG", "LTGTQ", LinkType::Body, 0),
    ]);

    let result = query_reverse(&conn, "LTGTQ").expect("Query should succeed");

    assert_eq!(result.references.len(), 1, "Should skip missing document");
    assert_eq!(result.references[0].document.id, "LSRCL");
}

#[test]
fn query_forward_handles_document_with_many_links() {
    let conn = create_test_db();
    let mut ids = vec!["LSRCM"];
    for i in 0..50 {
        ids.push(Box::leak(format!("LTGT{i:02}").into_boxed_str()));
    }
    setup_documents(&conn, &ids);

    let links: Vec<_> = (0..50).map(|i| ("LSRCM", ids[i + 1], LinkType::Body, i as u32)).collect();
    insert_links(&conn, links);

    let result = query_forward(&conn, "LSRCM").expect("Query should succeed");

    assert_eq!(result.references.len(), 50, "Should handle 50 references");
}

#[test]
fn query_reverse_handles_document_with_many_backlinks() {
    let conn = create_test_db();
    let mut ids = vec!["LTGTRR"];
    for i in 0..50 {
        ids.push(Box::leak(format!("LSRC{i:02}").into_boxed_str()));
    }
    setup_documents(&conn, &ids);

    let links: Vec<_> = (0..50).map(|i| (ids[i + 1], "LTGTRR", LinkType::Body, 0)).collect();
    insert_links(&conn, links);

    let result = query_reverse(&conn, "LTGTRR").expect("Query should succeed");

    assert_eq!(result.references.len(), 50, "Should handle 50 backlinks");
}
