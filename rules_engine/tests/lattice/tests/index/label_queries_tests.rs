use lattice::index::document_types::InsertDocument;
use lattice::index::label_queries::{
    add, add_to_multiple, count_documents_with_label, count_labels, delete_for_document,
    find_by_label, find_by_labels_all, find_by_labels_any, get_labels, has_label, label_exists,
    list_all, remove, remove_from_multiple, sync_labels, total_count,
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
    )
}

fn setup_documents(conn: &Connection, ids: &[&str]) {
    for id in ids {
        let doc = create_test_document(id, &format!("{id}.md"), id);
        document_queries::insert(conn, &doc).expect("Failed to insert document");
    }
}

#[test]
fn add_label_succeeds() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC1"]);

    let added = add(&conn, "LDOC1", "bug").expect("Add label should succeed");

    assert!(added, "Label should be added");
    assert!(has_label(&conn, "LDOC1", "bug").expect("Check should succeed"));
}

#[test]
fn add_label_idempotent() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC2"]);

    let first = add(&conn, "LDOC2", "feature").expect("Add label should succeed");
    let second = add(&conn, "LDOC2", "feature").expect("Add label should succeed");

    assert!(first, "First add should return true");
    assert!(!second, "Second add should return false (already exists)");
}

#[test]
fn remove_label_succeeds() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC3"]);
    add(&conn, "LDOC3", "urgent").expect("Add should succeed");

    let removed = remove(&conn, "LDOC3", "urgent").expect("Remove should succeed");

    assert!(removed, "Label should be removed");
    assert!(!has_label(&conn, "LDOC3", "urgent").expect("Check should succeed"));
}

#[test]
fn remove_nonexistent_label_returns_false() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC4"]);

    let removed = remove(&conn, "LDOC4", "nonexistent").expect("Remove should not error");

    assert!(!removed, "Remove should return false for nonexistent label");
}

#[test]
fn has_label_returns_false_for_missing() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC5"]);

    let result = has_label(&conn, "LDOC5", "missing").expect("Check should succeed");

    assert!(!result, "Document should not have label");
}

#[test]
fn get_labels_returns_sorted_list() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC6"]);
    add(&conn, "LDOC6", "zebra").expect("Add should succeed");
    add(&conn, "LDOC6", "alpha").expect("Add should succeed");
    add(&conn, "LDOC6", "beta").expect("Add should succeed");

    let labels = get_labels(&conn, "LDOC6").expect("Get labels should succeed");

    assert_eq!(labels, vec!["alpha", "beta", "zebra"], "Labels should be sorted alphabetically");
}

#[test]
fn get_labels_returns_empty_for_no_labels() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC7"]);

    let labels = get_labels(&conn, "LDOC7").expect("Get labels should succeed");

    assert!(labels.is_empty(), "Should have no labels");
}

#[test]
fn find_by_label_returns_matching_documents() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC8", "LDOC9", "LDOCA"]);
    add(&conn, "LDOC8", "important").expect("Add should succeed");
    add(&conn, "LDOC9", "important").expect("Add should succeed");
    add(&conn, "LDOCA", "other").expect("Add should succeed");

    let docs = find_by_label(&conn, "important").expect("Find should succeed");

    assert_eq!(docs.len(), 2, "Should find 2 documents with 'important' label");
    assert!(docs.contains(&"LDOC8".to_string()));
    assert!(docs.contains(&"LDOC9".to_string()));
}

#[test]
fn find_by_label_returns_empty_for_unknown_label() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCB"]);

    let docs = find_by_label(&conn, "unknown").expect("Find should succeed");

    assert!(docs.is_empty(), "Should find no documents");
}

#[test]
fn find_by_labels_all_returns_intersection() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCC", "LDOCD", "LDOCE"]);
    add(&conn, "LDOCC", "a").expect("Add should succeed");
    add(&conn, "LDOCC", "b").expect("Add should succeed");
    add(&conn, "LDOCD", "a").expect("Add should succeed");
    add(&conn, "LDOCE", "a").expect("Add should succeed");
    add(&conn, "LDOCE", "b").expect("Add should succeed");

    let docs = find_by_labels_all(&conn, &["a", "b"]).expect("Find should succeed");

    assert_eq!(docs.len(), 2, "Should find 2 documents with both labels");
    assert!(docs.contains(&"LDOCC".to_string()));
    assert!(docs.contains(&"LDOCE".to_string()));
    assert!(!docs.contains(&"LDOCD".to_string()), "LDOCD only has 'a'");
}

#[test]
fn find_by_labels_all_empty_input_returns_empty() {
    let conn = create_test_db();

    let docs = find_by_labels_all(&conn, &[]).expect("Find should succeed");

    assert!(docs.is_empty(), "Empty labels should return empty result");
}

#[test]
fn find_by_labels_any_returns_union() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCF", "LDOCG", "LDOCH"]);
    add(&conn, "LDOCF", "x").expect("Add should succeed");
    add(&conn, "LDOCG", "y").expect("Add should succeed");
    add(&conn, "LDOCH", "z").expect("Add should succeed");

    let docs = find_by_labels_any(&conn, &["x", "y"]).expect("Find should succeed");

    assert_eq!(docs.len(), 2, "Should find 2 documents with either label");
    assert!(docs.contains(&"LDOCF".to_string()));
    assert!(docs.contains(&"LDOCG".to_string()));
    assert!(!docs.contains(&"LDOCH".to_string()), "LDOCH only has 'z'");
}

#[test]
fn find_by_labels_any_empty_input_returns_empty() {
    let conn = create_test_db();

    let docs = find_by_labels_any(&conn, &[]).expect("Find should succeed");

    assert!(docs.is_empty(), "Empty labels should return empty result");
}

#[test]
fn list_all_returns_labels_with_counts() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCI", "LDOCJ", "LDOCK"]);
    add(&conn, "LDOCI", "common").expect("Add should succeed");
    add(&conn, "LDOCJ", "common").expect("Add should succeed");
    add(&conn, "LDOCK", "common").expect("Add should succeed");
    add(&conn, "LDOCI", "rare").expect("Add should succeed");

    let all = list_all(&conn).expect("List all should succeed");

    assert_eq!(all.len(), 2, "Should have 2 unique labels");
    let common = all.iter().find(|l| l.label == "common").expect("Should find 'common'");
    assert_eq!(common.count, 3, "'common' should have count 3");
    let rare = all.iter().find(|l| l.label == "rare").expect("Should find 'rare'");
    assert_eq!(rare.count, 1, "'rare' should have count 1");
}

#[test]
fn list_all_returns_empty_for_no_labels() {
    let conn = create_test_db();

    let all = list_all(&conn).expect("List all should succeed");

    assert!(all.is_empty(), "Should have no labels");
}

#[test]
fn add_to_multiple_adds_label_to_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCL", "LDOCM", "LDOCN"]);

    let added =
        add_to_multiple(&conn, &["LDOCL", "LDOCM", "LDOCN"], "batch").expect("Should succeed");

    assert_eq!(added, 3, "Should add label to 3 documents");
    assert!(has_label(&conn, "LDOCL", "batch").expect("Check should succeed"));
    assert!(has_label(&conn, "LDOCM", "batch").expect("Check should succeed"));
    assert!(has_label(&conn, "LDOCN", "batch").expect("Check should succeed"));
}

#[test]
fn add_to_multiple_skips_existing() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCO", "LDOCP"]);
    add(&conn, "LDOCO", "preexisting").expect("Add should succeed");

    let added = add_to_multiple(&conn, &["LDOCO", "LDOCP"], "preexisting").expect("Should succeed");

    assert_eq!(added, 1, "Should only add label to 1 document (LDOCP)");
}

#[test]
fn add_to_multiple_empty_list_returns_zero() {
    let conn = create_test_db();

    let added = add_to_multiple(&conn, &[], "label").expect("Should succeed");

    assert_eq!(added, 0, "Empty list should return 0");
}

#[test]
fn remove_from_multiple_removes_from_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCQ", "LDOCR"]);
    add(&conn, "LDOCQ", "toremove").expect("Add should succeed");
    add(&conn, "LDOCR", "toremove").expect("Add should succeed");

    let removed =
        remove_from_multiple(&conn, &["LDOCQ", "LDOCR"], "toremove").expect("Should succeed");

    assert_eq!(removed, 2, "Should remove from 2 documents");
    assert!(!has_label(&conn, "LDOCQ", "toremove").expect("Check should succeed"));
    assert!(!has_label(&conn, "LDOCR", "toremove").expect("Check should succeed"));
}

#[test]
fn remove_from_multiple_empty_list_returns_zero() {
    let conn = create_test_db();

    let removed = remove_from_multiple(&conn, &[], "label").expect("Should succeed");

    assert_eq!(removed, 0, "Empty list should return 0");
}

#[test]
fn sync_labels_replaces_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCS"]);
    add(&conn, "LDOCS", "old1").expect("Add should succeed");
    add(&conn, "LDOCS", "old2").expect("Add should succeed");

    sync_labels(&conn, "LDOCS", &["new1", "new2", "new3"]).expect("Sync should succeed");

    let labels = get_labels(&conn, "LDOCS").expect("Get labels should succeed");
    assert_eq!(labels, vec!["new1", "new2", "new3"], "Should have only new labels");
    assert!(!has_label(&conn, "LDOCS", "old1").expect("Check should succeed"));
    assert!(!has_label(&conn, "LDOCS", "old2").expect("Check should succeed"));
}

#[test]
fn sync_labels_empty_clears_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCT"]);
    add(&conn, "LDOCT", "label1").expect("Add should succeed");
    add(&conn, "LDOCT", "label2").expect("Add should succeed");

    sync_labels(&conn, "LDOCT", &[]).expect("Sync should succeed");

    let labels = get_labels(&conn, "LDOCT").expect("Get labels should succeed");
    assert!(labels.is_empty(), "Should have no labels after sync with empty list");
}

#[test]
fn delete_for_document_removes_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCU"]);
    add(&conn, "LDOCU", "a").expect("Add should succeed");
    add(&conn, "LDOCU", "b").expect("Add should succeed");
    add(&conn, "LDOCU", "c").expect("Add should succeed");

    let deleted = delete_for_document(&conn, "LDOCU").expect("Delete should succeed");

    assert_eq!(deleted, 3, "Should delete 3 labels");
    let labels = get_labels(&conn, "LDOCU").expect("Get labels should succeed");
    assert!(labels.is_empty(), "Should have no labels");
}

#[test]
fn delete_for_document_returns_zero_for_no_labels() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCV"]);

    let deleted = delete_for_document(&conn, "LDOCV").expect("Delete should succeed");

    assert_eq!(deleted, 0, "Should delete 0 labels");
}

#[test]
fn count_labels_returns_correct_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCW"]);
    add(&conn, "LDOCW", "a").expect("Add should succeed");
    add(&conn, "LDOCW", "b").expect("Add should succeed");

    let count = count_labels(&conn, "LDOCW").expect("Count should succeed");

    assert_eq!(count, 2, "Should have 2 labels");
}

#[test]
fn count_documents_with_label_returns_correct_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCX", "LDOCY", "LDOCZ"]);
    add(&conn, "LDOCX", "shared").expect("Add should succeed");
    add(&conn, "LDOCY", "shared").expect("Add should succeed");

    let count = count_documents_with_label(&conn, "shared").expect("Count should succeed");

    assert_eq!(count, 2, "Should have 2 documents with label");
}

#[test]
fn label_exists_returns_true_when_present() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC10"]);
    add(&conn, "LDOC10", "exists").expect("Add should succeed");

    let result = label_exists(&conn, "exists").expect("Check should succeed");

    assert!(result, "Label should exist");
}

#[test]
fn label_exists_returns_false_when_absent() {
    let conn = create_test_db();

    let result = label_exists(&conn, "absent").expect("Check should succeed");

    assert!(!result, "Label should not exist");
}

#[test]
fn total_count_returns_all_assignments() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC11", "LDOC12"]);
    add(&conn, "LDOC11", "a").expect("Add should succeed");
    add(&conn, "LDOC11", "b").expect("Add should succeed");
    add(&conn, "LDOC12", "a").expect("Add should succeed");

    let count = total_count(&conn).expect("Count should succeed");

    assert_eq!(count, 3, "Should have 3 total label assignments");
}
