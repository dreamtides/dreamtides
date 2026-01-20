use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{
    InsertLink, LinkType, count_incoming, count_outgoing, delete_by_source,
    delete_by_source_and_target, delete_by_target, exists, find_orphan_sources, get_source_ids,
    get_target_ids, insert_for_document, query_incoming, query_incoming_by_type, query_outgoing,
    query_outgoing_by_type,
};
use lattice::index::{document_queries, schema_definition};
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
        false,
    )
}

fn setup_documents(conn: &Connection, ids: &[&str]) {
    for id in ids {
        let doc = create_test_document(id, &format!("{id}.md"), id);
        document_queries::insert(conn, &doc).expect("Failed to insert document");
    }
}

#[test]
fn insert_links_for_document_succeeds() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC1", "LTGT1", "LTGT2"]);

    let links = vec![
        InsertLink {
            source_id: "LSRC1",
            target_id: "LTGT1",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRC1",
            target_id: "LTGT2",
            link_type: LinkType::Body,
            position: 1,
        },
    ];

    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let outgoing = query_outgoing(&conn, "LSRC1").expect("Query should succeed");
    assert_eq!(outgoing.len(), 2, "Should have 2 outgoing links");
}

#[test]
fn insert_empty_links_succeeds() {
    let conn = create_test_db();

    insert_for_document(&conn, &[]).expect("Insert empty links should succeed");
}

#[test]
fn insert_links_different_types() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC2", "LTGT3", "LTGT4", "LTGT5"]);

    let links = vec![
        InsertLink {
            source_id: "LSRC2",
            target_id: "LTGT3",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRC2",
            target_id: "LTGT4",
            link_type: LinkType::BlockedBy,
            position: 1,
        },
        InsertLink {
            source_id: "LSRC2",
            target_id: "LTGT5",
            link_type: LinkType::Blocking,
            position: 2,
        },
    ];

    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let all_outgoing = query_outgoing(&conn, "LSRC2").expect("Query should succeed");
    assert_eq!(all_outgoing.len(), 3, "Should have 3 total outgoing links");

    let body_only =
        query_outgoing_by_type(&conn, "LSRC2", LinkType::Body).expect("Query should succeed");
    assert_eq!(body_only.len(), 1, "Should have 1 body link");

    let blocked_by =
        query_outgoing_by_type(&conn, "LSRC2", LinkType::BlockedBy).expect("Query should succeed");
    assert_eq!(blocked_by.len(), 1, "Should have 1 blocked_by link");
}

#[test]
fn delete_by_source_removes_all_outgoing() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC3", "LTGT6", "LTGT7"]);

    let links = vec![
        InsertLink {
            source_id: "LSRC3",
            target_id: "LTGT6",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRC3",
            target_id: "LTGT7",
            link_type: LinkType::Body,
            position: 1,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let deleted = delete_by_source(&conn, "LSRC3").expect("Delete should succeed");

    assert_eq!(deleted, 2, "Should delete 2 links");
    let outgoing = query_outgoing(&conn, "LSRC3").expect("Query should succeed");
    assert!(outgoing.is_empty(), "Should have no outgoing links after delete");
}

#[test]
fn delete_by_source_nonexistent_returns_zero() {
    let conn = create_test_db();

    let deleted = delete_by_source(&conn, "LNONE").expect("Delete should not error");

    assert_eq!(deleted, 0, "Should delete 0 links for nonexistent source");
}

#[test]
fn delete_by_target_removes_all_incoming() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC4", "LSRC5", "LTGT8"]);

    let links1 = vec![InsertLink {
        source_id: "LSRC4",
        target_id: "LTGT8",
        link_type: LinkType::Body,
        position: 0,
    }];
    let links2 = vec![InsertLink {
        source_id: "LSRC5",
        target_id: "LTGT8",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links1).expect("Insert links should succeed");
    insert_for_document(&conn, &links2).expect("Insert links should succeed");

    let deleted = delete_by_target(&conn, "LTGT8").expect("Delete should succeed");

    assert_eq!(deleted, 2, "Should delete 2 incoming links");
    let incoming = query_incoming(&conn, "LTGT8").expect("Query should succeed");
    assert!(incoming.is_empty(), "Should have no incoming links after delete");
}

#[test]
fn delete_by_source_and_target_removes_specific_links() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC6", "LTGT9", "LTGTA"]);

    let links = vec![
        InsertLink {
            source_id: "LSRC6",
            target_id: "LTGT9",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRC6",
            target_id: "LTGTA",
            link_type: LinkType::Body,
            position: 1,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let deleted =
        delete_by_source_and_target(&conn, "LSRC6", "LTGT9").expect("Delete should succeed");

    assert_eq!(deleted, 1, "Should delete 1 link");
    let outgoing = query_outgoing(&conn, "LSRC6").expect("Query should succeed");
    assert_eq!(outgoing.len(), 1, "Should have 1 remaining link");
    assert_eq!(outgoing[0].target_id, "LTGTA", "Remaining link should be to LTGTA");
}

#[test]
fn query_outgoing_returns_links_in_position_order() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC7", "LTGTB", "LTGTC", "LTGTD"]);

    let links = vec![
        InsertLink {
            source_id: "LSRC7",
            target_id: "LTGTC",
            link_type: LinkType::Body,
            position: 2,
        },
        InsertLink {
            source_id: "LSRC7",
            target_id: "LTGTB",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRC7",
            target_id: "LTGTD",
            link_type: LinkType::Body,
            position: 1,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let outgoing = query_outgoing(&conn, "LSRC7").expect("Query should succeed");

    assert_eq!(outgoing.len(), 3, "Should have 3 links");
    assert_eq!(outgoing[0].target_id, "LTGTB", "First link should be position 0");
    assert_eq!(outgoing[1].target_id, "LTGTD", "Second link should be position 1");
    assert_eq!(outgoing[2].target_id, "LTGTC", "Third link should be position 2");
}

#[test]
fn query_incoming_returns_backlinks() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRC8", "LSRC9", "LTGTE"]);

    let links1 = vec![InsertLink {
        source_id: "LSRC8",
        target_id: "LTGTE",
        link_type: LinkType::Body,
        position: 0,
    }];
    let links2 = vec![InsertLink {
        source_id: "LSRC9",
        target_id: "LTGTE",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links1).expect("Insert links should succeed");
    insert_for_document(&conn, &links2).expect("Insert links should succeed");

    let incoming = query_incoming(&conn, "LTGTE").expect("Query should succeed");

    assert_eq!(incoming.len(), 2, "Should have 2 incoming links");
    let source_ids: Vec<&str> = incoming.iter().map(|l| l.source_id.as_str()).collect();
    assert!(source_ids.contains(&"LSRC8"), "Should include LSRC8");
    assert!(source_ids.contains(&"LSRC9"), "Should include LSRC9");
}

#[test]
fn query_incoming_by_type_filters_correctly() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCA", "LSRCB", "LTGTF"]);

    let links1 = vec![InsertLink {
        source_id: "LSRCA",
        target_id: "LTGTF",
        link_type: LinkType::Body,
        position: 0,
    }];
    let links2 = vec![InsertLink {
        source_id: "LSRCB",
        target_id: "LTGTF",
        link_type: LinkType::BlockedBy,
        position: 0,
    }];
    insert_for_document(&conn, &links1).expect("Insert links should succeed");
    insert_for_document(&conn, &links2).expect("Insert links should succeed");

    let body_incoming =
        query_incoming_by_type(&conn, "LTGTF", LinkType::Body).expect("Query should succeed");
    let blocked_incoming =
        query_incoming_by_type(&conn, "LTGTF", LinkType::BlockedBy).expect("Query should succeed");

    assert_eq!(body_incoming.len(), 1, "Should have 1 body incoming link");
    assert_eq!(body_incoming[0].source_id, "LSRCA");
    assert_eq!(blocked_incoming.len(), 1, "Should have 1 blocked_by incoming link");
    assert_eq!(blocked_incoming[0].source_id, "LSRCB");
}

#[test]
fn exists_returns_true_for_existing_link() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCC", "LTGTG"]);

    let links = vec![InsertLink {
        source_id: "LSRCC",
        target_id: "LTGTG",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    assert!(exists(&conn, "LSRCC", "LTGTG").expect("Check should succeed"), "Link should exist");
}

#[test]
fn exists_returns_false_for_nonexistent_link() {
    let conn = create_test_db();

    assert!(
        !exists(&conn, "LNONE", "LNONE").expect("Check should succeed"),
        "Link should not exist"
    );
}

#[test]
fn count_outgoing_returns_correct_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCD", "LTGTH", "LTGTI", "LTGTJ"]);

    let links = vec![
        InsertLink {
            source_id: "LSRCD",
            target_id: "LTGTH",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRCD",
            target_id: "LTGTI",
            link_type: LinkType::Body,
            position: 1,
        },
        InsertLink {
            source_id: "LSRCD",
            target_id: "LTGTJ",
            link_type: LinkType::Body,
            position: 2,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let count = count_outgoing(&conn, "LSRCD").expect("Count should succeed");

    assert_eq!(count, 3, "Should count 3 outgoing links");
}

#[test]
fn count_incoming_returns_correct_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCE", "LSRCF", "LTGTK"]);

    let links1 = vec![InsertLink {
        source_id: "LSRCE",
        target_id: "LTGTK",
        link_type: LinkType::Body,
        position: 0,
    }];
    let links2 = vec![InsertLink {
        source_id: "LSRCF",
        target_id: "LTGTK",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links1).expect("Insert links should succeed");
    insert_for_document(&conn, &links2).expect("Insert links should succeed");

    let count = count_incoming(&conn, "LTGTK").expect("Count should succeed");

    assert_eq!(count, 2, "Should count 2 incoming links");
}

#[test]
fn find_orphan_sources_returns_documents_with_no_incoming_links() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCG", "LTGTL", "LTGTM"]);

    let links = vec![
        InsertLink {
            source_id: "LSRCG",
            target_id: "LTGTL",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LTGTL",
            target_id: "LTGTM",
            link_type: LinkType::Body,
            position: 0,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let orphans = find_orphan_sources(&conn).expect("Find orphans should succeed");

    assert!(orphans.contains(&"LSRCG".to_string()), "LSRCG should be orphan (no incoming)");
    assert!(
        !orphans.contains(&"LTGTL".to_string()),
        "LTGTL should not be orphan (has incoming from LSRCG)"
    );
    assert!(
        !orphans.contains(&"LTGTM".to_string()),
        "LTGTM should not be orphan (has incoming from LTGTL)"
    );
}

#[test]
fn get_target_ids_returns_distinct_targets() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCH", "LTGTN", "LTGTO"]);

    let links = vec![
        InsertLink {
            source_id: "LSRCH",
            target_id: "LTGTN",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRCH",
            target_id: "LTGTN",
            link_type: LinkType::BlockedBy,
            position: 1,
        },
        InsertLink {
            source_id: "LSRCH",
            target_id: "LTGTO",
            link_type: LinkType::Body,
            position: 2,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let targets = get_target_ids(&conn, "LSRCH").expect("Get targets should succeed");

    assert_eq!(targets.len(), 2, "Should have 2 distinct targets");
    assert!(targets.contains(&"LTGTN".to_string()));
    assert!(targets.contains(&"LTGTO".to_string()));
}

#[test]
fn get_source_ids_returns_distinct_sources() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCI", "LSRCJ", "LTGTP"]);

    let links1 = vec![
        InsertLink {
            source_id: "LSRCI",
            target_id: "LTGTP",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRCI",
            target_id: "LTGTP",
            link_type: LinkType::Blocking,
            position: 1,
        },
    ];
    let links2 = vec![InsertLink {
        source_id: "LSRCJ",
        target_id: "LTGTP",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links1).expect("Insert links should succeed");
    insert_for_document(&conn, &links2).expect("Insert links should succeed");

    let sources = get_source_ids(&conn, "LTGTP").expect("Get sources should succeed");

    assert_eq!(sources.len(), 2, "Should have 2 distinct sources");
    assert!(sources.contains(&"LSRCI".to_string()));
    assert!(sources.contains(&"LSRCJ".to_string()));
}

#[test]
fn link_type_display_formats_correctly() {
    assert_eq!(format!("{}", LinkType::Body), "body");
    assert_eq!(format!("{}", LinkType::BlockedBy), "blocked_by");
    assert_eq!(format!("{}", LinkType::Blocking), "blocking");
    assert_eq!(format!("{}", LinkType::DiscoveredFrom), "discovered_from");
}

#[test]
fn link_row_preserves_all_fields() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCK", "LTGTQ"]);

    let links = vec![InsertLink {
        source_id: "LSRCK",
        target_id: "LTGTQ",
        link_type: LinkType::DiscoveredFrom,
        position: 42,
    }];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let outgoing = query_outgoing(&conn, "LSRCK").expect("Query should succeed");

    assert_eq!(outgoing.len(), 1);
    let link = &outgoing[0];
    assert_eq!(link.source_id, "LSRCK");
    assert_eq!(link.target_id, "LTGTQ");
    assert_eq!(link.link_type, LinkType::DiscoveredFrom);
    assert_eq!(link.position, 42);
}

#[test]
fn triggers_update_link_counts_on_insert() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCL", "LTGTR"]);

    let links = vec![InsertLink {
        source_id: "LSRCL",
        target_id: "LTGTR",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let source = document_queries::lookup_by_id(&conn, "LSRCL")
        .expect("Lookup should succeed")
        .expect("Document should exist");
    let target = document_queries::lookup_by_id(&conn, "LTGTR")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    assert_eq!(source.link_count, 1, "Source link_count should be 1");
    assert_eq!(target.backlink_count, 1, "Target backlink_count should be 1");
}

#[test]
fn triggers_update_link_counts_on_delete() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCM", "LTGTS"]);

    let links = vec![InsertLink {
        source_id: "LSRCM",
        target_id: "LTGTS",
        link_type: LinkType::Body,
        position: 0,
    }];
    insert_for_document(&conn, &links).expect("Insert links should succeed");
    delete_by_source(&conn, "LSRCM").expect("Delete should succeed");

    let source = document_queries::lookup_by_id(&conn, "LSRCM")
        .expect("Lookup should succeed")
        .expect("Document should exist");
    let target = document_queries::lookup_by_id(&conn, "LTGTS")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    assert_eq!(source.link_count, 0, "Source link_count should be 0 after delete");
    assert_eq!(target.backlink_count, 0, "Target backlink_count should be 0 after delete");
}

#[test]
fn multiple_links_between_same_documents_with_different_positions() {
    let conn = create_test_db();
    setup_documents(&conn, &["LSRCN", "LTGTT"]);

    let links = vec![
        InsertLink {
            source_id: "LSRCN",
            target_id: "LTGTT",
            link_type: LinkType::Body,
            position: 0,
        },
        InsertLink {
            source_id: "LSRCN",
            target_id: "LTGTT",
            link_type: LinkType::Body,
            position: 5,
        },
        InsertLink {
            source_id: "LSRCN",
            target_id: "LTGTT",
            link_type: LinkType::Body,
            position: 10,
        },
    ];
    insert_for_document(&conn, &links).expect("Insert links should succeed");

    let outgoing = query_outgoing(&conn, "LSRCN").expect("Query should succeed");

    assert_eq!(outgoing.len(), 3, "Should allow multiple links between same documents");
    assert_eq!(outgoing[0].position, 0);
    assert_eq!(outgoing[1].position, 5);
    assert_eq!(outgoing[2].position, 10);
}
