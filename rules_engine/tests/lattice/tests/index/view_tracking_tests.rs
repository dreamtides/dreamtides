use lattice::index::document_types::InsertDocument;
use lattice::index::view_tracking::{
    delete_view, get_last_viewed, get_view_count, get_view_data, get_view_stats, record_view,
    reset_all_views,
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
fn record_view_creates_new_entry() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC1"]);

    let count = record_view(&conn, "LDOC1").expect("Record view should succeed");

    assert_eq!(count, 1, "First view should result in count of 1");
}

#[test]
fn record_view_increments_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC2"]);

    record_view(&conn, "LDOC2").expect("First view should succeed");
    record_view(&conn, "LDOC2").expect("Second view should succeed");
    let count = record_view(&conn, "LDOC2").expect("Third view should succeed");

    assert_eq!(count, 3, "Third view should result in count of 3");
}

#[test]
fn get_view_count_returns_zero_for_unviewed() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC3"]);

    let count = get_view_count(&conn, "LDOC3").expect("Get view count should succeed");

    assert_eq!(count, 0, "Unviewed document should have count 0");
}

#[test]
fn get_view_count_returns_correct_count() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC4"]);
    record_view(&conn, "LDOC4").expect("View should succeed");
    record_view(&conn, "LDOC4").expect("View should succeed");

    let count = get_view_count(&conn, "LDOC4").expect("Get view count should succeed");

    assert_eq!(count, 2, "Should return correct view count");
}

#[test]
fn get_last_viewed_returns_none_for_unviewed() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC5"]);

    let last_viewed = get_last_viewed(&conn, "LDOC5").expect("Get last viewed should succeed");

    assert!(last_viewed.is_none(), "Unviewed document should have no last_viewed timestamp");
}

#[test]
fn get_last_viewed_returns_timestamp_after_view() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC6"]);
    record_view(&conn, "LDOC6").expect("View should succeed");

    let last_viewed = get_last_viewed(&conn, "LDOC6").expect("Get last viewed should succeed");

    assert!(last_viewed.is_some(), "Viewed document should have last_viewed timestamp");
}

#[test]
fn get_view_data_returns_none_for_unviewed() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC7"]);

    let data = get_view_data(&conn, "LDOC7").expect("Get view data should succeed");

    assert!(data.is_none(), "Unviewed document should have no view data");
}

#[test]
fn get_view_data_returns_complete_data() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC8"]);
    record_view(&conn, "LDOC8").expect("View should succeed");
    record_view(&conn, "LDOC8").expect("View should succeed");

    let data = get_view_data(&conn, "LDOC8").expect("Get view data should succeed");

    let view_data = data.expect("Should have view data");
    assert_eq!(view_data.document_id, "LDOC8");
    assert_eq!(view_data.view_count, 2);
}

#[test]
fn get_view_stats_returns_zeros_initially() {
    let conn = create_test_db();

    let stats = get_view_stats(&conn).expect("Get view stats should succeed");

    assert_eq!(stats.tracked_documents, 0, "No documents tracked initially");
    assert_eq!(stats.total_views, 0, "No views initially");
}

#[test]
fn get_view_stats_returns_aggregate_counts() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOC9", "LDOCA", "LDOCB"]);
    record_view(&conn, "LDOC9").expect("View should succeed");
    record_view(&conn, "LDOC9").expect("View should succeed");
    record_view(&conn, "LDOCA").expect("View should succeed");

    let stats = get_view_stats(&conn).expect("Get view stats should succeed");

    assert_eq!(stats.tracked_documents, 2, "Two documents have been viewed");
    assert_eq!(stats.total_views, 3, "Total of 3 views recorded");
}

#[test]
fn reset_all_views_clears_all_data() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCC", "LDOCD"]);
    record_view(&conn, "LDOCC").expect("View should succeed");
    record_view(&conn, "LDOCD").expect("View should succeed");

    let deleted = reset_all_views(&conn).expect("Reset should succeed");

    assert_eq!(deleted, 2, "Should delete 2 view records");
    assert_eq!(get_view_count(&conn, "LDOCC").expect("Count should succeed"), 0);
    assert_eq!(get_view_count(&conn, "LDOCD").expect("Count should succeed"), 0);
}

#[test]
fn reset_all_views_returns_zero_when_empty() {
    let conn = create_test_db();

    let deleted = reset_all_views(&conn).expect("Reset should succeed");

    assert_eq!(deleted, 0, "Should delete 0 records when no views exist");
}

#[test]
fn delete_view_removes_single_document() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCE", "LDOCF"]);
    record_view(&conn, "LDOCE").expect("View should succeed");
    record_view(&conn, "LDOCF").expect("View should succeed");

    let deleted = delete_view(&conn, "LDOCE").expect("Delete should succeed");

    assert!(deleted, "Should return true when view data deleted");
    assert_eq!(get_view_count(&conn, "LDOCE").expect("Count should succeed"), 0);
    assert_eq!(get_view_count(&conn, "LDOCF").expect("Count should succeed"), 1);
}

#[test]
fn delete_view_returns_false_for_unviewed() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCG"]);

    let deleted = delete_view(&conn, "LDOCG").expect("Delete should succeed");

    assert!(!deleted, "Should return false when no view data existed");
}

#[test]
fn documents_view_count_updated_by_trigger() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCH"]);

    // Record views
    record_view(&conn, "LDOCH").expect("View should succeed");
    record_view(&conn, "LDOCH").expect("View should succeed");

    // Check that documents.view_count was updated via trigger
    let denormalized_count: i32 = conn
        .query_row("SELECT view_count FROM documents WHERE id = ?", ["LDOCH"], |row| row.get(0))
        .expect("Query should succeed");

    assert_eq!(denormalized_count, 2, "Trigger should update documents.view_count");
}

#[test]
fn documents_view_count_reset_by_delete_trigger() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCI"]);
    record_view(&conn, "LDOCI").expect("View should succeed");
    record_view(&conn, "LDOCI").expect("View should succeed");

    delete_view(&conn, "LDOCI").expect("Delete should succeed");

    let denormalized_count: i32 = conn
        .query_row("SELECT view_count FROM documents WHERE id = ?", ["LDOCI"], |row| row.get(0))
        .expect("Query should succeed");

    assert_eq!(denormalized_count, 0, "Trigger should reset documents.view_count to 0");
}

#[test]
fn view_updates_last_viewed_timestamp() {
    let conn = create_test_db();
    setup_documents(&conn, &["LDOCJ"]);

    record_view(&conn, "LDOCJ").expect("First view should succeed");
    let first_timestamp = get_last_viewed(&conn, "LDOCJ")
        .expect("Get last viewed should succeed")
        .expect("Should have timestamp");

    // Small delay to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(10));

    record_view(&conn, "LDOCJ").expect("Second view should succeed");
    let second_timestamp = get_last_viewed(&conn, "LDOCJ")
        .expect("Get last viewed should succeed")
        .expect("Should have timestamp");

    assert!(
        second_timestamp >= first_timestamp,
        "Second view should update timestamp to later or equal time"
    );
}
