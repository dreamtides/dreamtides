use lattice::index::document_types::InsertDocument;
use lattice::index::fulltext_search::{
    clear_index, count, exists, index_batch, index_document, remove_document, search,
    search_with_limit, search_with_snippets,
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
fn index_document_adds_to_fts() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC1"]);

    index_document(&conn, "LDOC1", "This is searchable content").expect("Index should succeed");

    assert!(exists(&conn, "LDOC1").expect("Exists check should succeed"));
    assert_eq!(count(&conn).expect("Count should succeed"), 1);
}

#[test]
fn index_document_replaces_existing() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC2"]);

    index_document(&conn, "LDOC2", "Original content").expect("First index should succeed");
    index_document(&conn, "LDOC2", "Updated content").expect("Second index should succeed");

    assert_eq!(count(&conn).expect("Count should succeed"), 1, "Should not duplicate entries");
    let results = search(&conn, "Updated").expect("Search should succeed");
    assert_eq!(results.len(), 1, "Should find updated content");
}

#[test]
fn remove_document_deletes_from_fts() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC3"]);
    index_document(&conn, "LDOC3", "Some content").expect("Index should succeed");

    let removed = remove_document(&conn, "LDOC3").expect("Remove should succeed");

    assert!(removed, "Should return true when document existed");
    assert!(!exists(&conn, "LDOC3").expect("Exists check should succeed"));
}

#[test]
fn remove_nonexistent_returns_false() {
    let conn = create_test_db();

    let removed = remove_document(&conn, "NONEXISTENT").expect("Remove should not error");

    assert!(!removed, "Should return false for nonexistent document");
}

#[test]
fn search_finds_single_word() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC4", "LDOC5"]);
    index_document(&conn, "LDOC4", "The error occurred during login")
        .expect("Index should succeed");
    index_document(&conn, "LDOC5", "Everything worked correctly").expect("Index should succeed");

    let results = search(&conn, "error").expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find one document with 'error'");
    assert_eq!(results[0].document_id, "LDOC4");
}

#[test]
fn search_empty_query_returns_empty() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC6"]);
    index_document(&conn, "LDOC6", "Some content").expect("Index should succeed");

    let results = search(&conn, "").expect("Search should succeed");
    let results_whitespace = search(&conn, "   ").expect("Search should succeed");

    assert!(results.is_empty(), "Empty query should return no results");
    assert!(results_whitespace.is_empty(), "Whitespace query should return no results");
}

#[test]
fn search_phrase_match() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC7", "LDOC8"]);
    index_document(&conn, "LDOC7", "Fix the login bug immediately").expect("Index should succeed");
    index_document(&conn, "LDOC8", "Login and bug are separate words")
        .expect("Index should succeed");

    let results = search(&conn, "\"login bug\"").expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should only find exact phrase");
    assert_eq!(results[0].document_id, "LDOC7");
}

#[test]
fn search_and_operator() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC9", "LDOCA", "LDOCB"]);
    index_document(&conn, "LDOC9", "Error in the login system").expect("Index should succeed");
    index_document(&conn, "LDOCA", "Just an error message").expect("Index should succeed");
    index_document(&conn, "LDOCB", "Login was successful").expect("Index should succeed");

    let results = search(&conn, "error AND login").expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find document with both terms");
    assert_eq!(results[0].document_id, "LDOC9");
}

#[test]
fn search_or_operator() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCC", "LDOCD", "LDOCE"]);
    index_document(&conn, "LDOCC", "An error occurred").expect("Index should succeed");
    index_document(&conn, "LDOCD", "A warning was issued").expect("Index should succeed");
    index_document(&conn, "LDOCE", "Everything normal").expect("Index should succeed");

    let results = search(&conn, "error OR warning").expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should find documents with either term");
    let ids: Vec<_> = results.iter().map(|r| r.document_id.as_str()).collect();
    assert!(ids.contains(&"LDOCC"));
    assert!(ids.contains(&"LDOCD"));
}

#[test]
fn search_not_operator() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCF", "LDOCG"]);
    index_document(&conn, "LDOCF", "Error in production code").expect("Index should succeed");
    index_document(&conn, "LDOCG", "Error in test code").expect("Index should succeed");

    let results = search(&conn, "error NOT test").expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find document without excluded term");
    assert_eq!(results[0].document_id, "LDOCF");
}

#[test]
fn search_prefix_match() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCH", "LDOCI", "LDOCJ"]);
    index_document(&conn, "LDOCH", "Authentication failed").expect("Index should succeed");
    index_document(&conn, "LDOCI", "Authorization denied").expect("Index should succeed");
    index_document(&conn, "LDOCJ", "Logging enabled").expect("Index should succeed");

    let results = search(&conn, "auth*").expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should find documents starting with 'auth'");
    let ids: Vec<_> = results.iter().map(|r| r.document_id.as_str()).collect();
    assert!(ids.contains(&"LDOCH"));
    assert!(ids.contains(&"LDOCI"));
}

#[test]
fn search_near_operator() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCK", "LDOCL"]);
    index_document(&conn, "LDOCK", "The error occurred during login")
        .expect("Index should succeed");
    index_document(&conn, "LDOCL", "The error was in a completely different module than login")
        .expect("Index should succeed");

    let results = search(&conn, "NEAR(error login, 3)").expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find document with terms within 3 words");
    assert_eq!(results[0].document_id, "LDOCK");
}

#[test]
fn search_with_limit_respects_limit() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCM", "LDOCN", "LDOCO"]);
    index_document(&conn, "LDOCM", "Match one").expect("Index should succeed");
    index_document(&conn, "LDOCN", "Match two").expect("Index should succeed");
    index_document(&conn, "LDOCO", "Match three").expect("Index should succeed");

    let results = search_with_limit(&conn, "Match", Some(2)).expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should respect limit of 2");
}

#[test]
fn search_returns_bm25_rank() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCP", "LDOCQ"]);
    index_document(&conn, "LDOCP", "error error error error").expect("Index should succeed");
    index_document(&conn, "LDOCQ", "error").expect("Index should succeed");

    let results = search(&conn, "error").expect("Search should succeed");

    assert_eq!(results.len(), 2);
    // BM25 returns negative values, more relevant = more negative
    // Document with more occurrences should have lower (more negative) rank
    assert!(
        results[0].rank < results[1].rank,
        "Higher frequency should rank higher (more negative)"
    );
    assert_eq!(results[0].document_id, "LDOCP", "Document with more matches should be first");
}

#[test]
fn search_with_snippets_returns_context() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCR"]);
    index_document(&conn, "LDOCR", "Before the important keyword there was text")
        .expect("Index should succeed");

    let results = search_with_snippets(&conn, "keyword", "<mark>", "</mark>", None)
        .expect("Search should succeed");

    assert_eq!(results.len(), 1);
    assert!(results[0].snippet.contains("<mark>"), "Snippet should contain highlight start marker");
    assert!(results[0].snippet.contains("</mark>"), "Snippet should contain highlight end marker");
    assert!(results[0].snippet.contains("keyword"), "Snippet should contain the matched word");
}

#[test]
fn search_with_snippets_handles_no_matches() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCS"]);
    index_document(&conn, "LDOCS", "Some content").expect("Index should succeed");

    let results =
        search_with_snippets(&conn, "nonexistent", "*", "*", None).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty for no matches");
}

#[test]
fn index_batch_indexes_multiple() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCT", "LDOCU", "LDOCV"]);

    let docs =
        [("LDOCT", "First content"), ("LDOCU", "Second content"), ("LDOCV", "Third content")];
    let indexed = index_batch(&conn, &docs).expect("Batch index should succeed");

    assert_eq!(indexed, 3, "Should index 3 documents");
    assert_eq!(count(&conn).expect("Count should succeed"), 3);
    assert!(exists(&conn, "LDOCT").expect("Exists check should succeed"));
    assert!(exists(&conn, "LDOCU").expect("Exists check should succeed"));
    assert!(exists(&conn, "LDOCV").expect("Exists check should succeed"));
}

#[test]
fn index_batch_empty_returns_zero() {
    let conn = create_test_db();

    let indexed = index_batch(&conn, &[]).expect("Batch index should succeed");

    assert_eq!(indexed, 0, "Empty batch should return 0");
}

#[test]
fn index_batch_replaces_existing() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCW"]);
    index_document(&conn, "LDOCW", "Original").expect("Index should succeed");

    let docs = [("LDOCW", "Updated")];
    index_batch(&conn, &docs).expect("Batch index should succeed");

    assert_eq!(count(&conn).expect("Count should succeed"), 1, "Should not duplicate");
    let results = search(&conn, "Updated").expect("Search should succeed");
    assert_eq!(results.len(), 1, "Should find updated content");
}

#[test]
fn clear_index_removes_all() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCX", "LDOCY"]);
    index_document(&conn, "LDOCX", "Content one").expect("Index should succeed");
    index_document(&conn, "LDOCY", "Content two").expect("Index should succeed");

    clear_index(&conn).expect("Clear should succeed");

    assert_eq!(count(&conn).expect("Count should succeed"), 0);
}

#[test]
fn count_returns_zero_for_empty_index() {
    let conn = create_test_db();

    let result = count(&conn).expect("Count should succeed");

    assert_eq!(result, 0, "Empty index should have count 0");
}

#[test]
fn exists_returns_false_for_missing() {
    let conn = create_test_db();

    let result = exists(&conn, "NONEXISTENT").expect("Exists check should succeed");

    assert!(!result, "Should return false for nonexistent document");
}

#[test]
fn document_delete_trigger_removes_fts_entry() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCZ"]);
    index_document(&conn, "LDOCZ", "Content to be deleted").expect("Index should succeed");
    assert!(exists(&conn, "LDOCZ").expect("Should exist before delete"));

    // Delete the document from the documents table - trigger should clean up FTS
    document_queries::delete_by_id(&conn, "LDOCZ").expect("Delete should succeed");

    assert!(
        !exists(&conn, "LDOCZ").expect("Exists check should succeed"),
        "FTS entry should be removed by trigger"
    );
}

#[test]
fn search_case_insensitive() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC10"]);
    index_document(&conn, "LDOC10", "ERROR in uppercase").expect("Index should succeed");

    let results_lower = search(&conn, "error").expect("Search should succeed");
    let results_upper = search(&conn, "ERROR").expect("Search should succeed");
    let results_mixed = search(&conn, "ErRoR").expect("Search should succeed");

    assert_eq!(results_lower.len(), 1, "Lowercase query should match");
    assert_eq!(results_upper.len(), 1, "Uppercase query should match");
    assert_eq!(results_mixed.len(), 1, "Mixed case query should match");
}
