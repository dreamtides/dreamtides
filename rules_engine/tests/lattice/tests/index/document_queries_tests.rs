use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_filter::{DocumentFilter, DocumentState, SortColumn, SortOrder};
use lattice::index::document_queries::{
    all_ids, all_paths, count, delete_batch, delete_by_id, delete_by_path_prefix, exists,
    exists_at_path, insert, insert_batch, lookup_by_id, lookup_by_name, lookup_by_path, query,
    update, update_batch,
};
use lattice::index::document_types::{InsertDocument, UpdateBuilder};
use lattice::index::schema_definition;
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

fn create_task_document(id: &str, path: &str, name: &str, priority: u8) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Task: {name}"),
        Some(TaskType::Task),
        Some(priority),
        None,
        None,
        None,
        "def456".to_string(),
        200,
        false,
    )
}

#[test]
fn insert_document_succeeds() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");

    insert(&conn, &doc).expect("Insert should succeed");

    let result = lookup_by_id(&conn, "LDOC01").expect("Lookup should succeed");
    assert!(result.is_some(), "Document should exist after insert");
    let row = result.unwrap();
    assert_eq!(row.id, "LDOC01");
    assert_eq!(row.path, "api/api.md");
    assert_eq!(row.name, "api");
}

#[test]
fn insert_document_sets_is_root_correctly() {
    let conn = create_test_db();
    let root_doc = create_test_document("LROOT", "api/api.md", "api");
    let non_root = create_test_document("LCHLD", "api/child.md", "child");

    insert(&conn, &root_doc).expect("Insert root should succeed");
    insert(&conn, &non_root).expect("Insert child should succeed");

    let root_result = lookup_by_id(&conn, "LROOT").expect("Lookup root should succeed").unwrap();
    let child_result = lookup_by_id(&conn, "LCHLD").expect("Lookup child should succeed").unwrap();

    assert!(root_result.is_root, "api/api.md should be marked as root");
    assert!(!child_result.is_root, "api/child.md should not be marked as root");
}

#[test]
fn insert_document_sets_directory_flags() {
    let conn = create_test_db();
    let task = create_test_document("LTASK", "api/tasks/fix_bug.md", "fix-bug");
    let doc = create_test_document("LDOCS", "api/docs/design.md", "design");
    let other = create_test_document("LOTHR", "api/other.md", "other");

    insert(&conn, &task).expect("Insert task should succeed");
    insert(&conn, &doc).expect("Insert doc should succeed");
    insert(&conn, &other).expect("Insert other should succeed");

    let task_row = lookup_by_id(&conn, "LTASK").expect("Lookup should succeed").unwrap();
    let doc_row = lookup_by_id(&conn, "LDOCS").expect("Lookup should succeed").unwrap();
    let other_row = lookup_by_id(&conn, "LOTHR").expect("Lookup should succeed").unwrap();

    assert!(task_row.in_tasks_dir, "Task should be in tasks dir");
    assert!(!task_row.in_docs_dir, "Task should not be in docs dir");

    assert!(!doc_row.in_tasks_dir, "Doc should not be in tasks dir");
    assert!(doc_row.in_docs_dir, "Doc should be in docs dir");

    assert!(!other_row.in_tasks_dir, "Other should not be in tasks dir");
    assert!(!other_row.in_docs_dir, "Other should not be in docs dir");
}

#[test]
fn insert_document_sets_is_closed() {
    let conn = create_test_db();
    let open_task = create_test_document("LOPEN", "api/tasks/open.md", "open");
    let closed_task = create_test_document("LCLSD", "api/tasks/.closed/done.md", "done");

    insert(&conn, &open_task).expect("Insert open should succeed");
    insert(&conn, &closed_task).expect("Insert closed should succeed");

    let open_row = lookup_by_id(&conn, "LOPEN").expect("Lookup should succeed").unwrap();
    let closed_row = lookup_by_id(&conn, "LCLSD").expect("Lookup should succeed").unwrap();

    assert!(!open_row.is_closed, "Open task should not be closed");
    assert!(closed_row.is_closed, "Closed task should be closed");
}

#[test]
fn insert_batch_succeeds() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LDOC1", "a.md", "a"),
        create_test_document("LDOC2", "b.md", "b"),
        create_test_document("LDOC3", "c.md", "c"),
    ];

    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    assert!(exists(&conn, "LDOC1").expect("Check should succeed"), "Doc 1 should exist");
    assert!(exists(&conn, "LDOC2").expect("Check should succeed"), "Doc 2 should exist");
    assert!(exists(&conn, "LDOC3").expect("Check should succeed"), "Doc 3 should exist");
}

#[test]
fn insert_batch_empty_succeeds() {
    let conn = create_test_db();

    insert_batch(&conn, &[]).expect("Empty batch insert should succeed");
}

#[test]
fn update_document_changes_fields() {
    let conn = create_test_db();
    let doc = create_test_document("LUPDT", "test.md", "test");
    insert(&conn, &doc).expect("Insert should succeed");

    let builder = UpdateBuilder::new()
        .path("new/path.md")
        .name("new-name")
        .description("New description")
        .is_closed(true)
        .priority(Some(1))
        .body_hash("newhash")
        .content_length(500);

    let updated = update(&conn, "LUPDT", &builder).expect("Update should succeed");

    assert!(updated, "Update should report success");

    let row = lookup_by_id(&conn, "LUPDT").expect("Lookup should succeed").unwrap();
    assert_eq!(row.path, "new/path.md");
    assert_eq!(row.name, "new-name");
    assert_eq!(row.description, "New description");
    assert!(row.is_closed);
    assert_eq!(row.priority, Some(1));
    assert_eq!(row.body_hash, "newhash");
    assert_eq!(row.content_length, 500);
}

#[test]
fn update_nonexistent_document_returns_false() {
    let conn = create_test_db();

    let builder = UpdateBuilder::new().path("path.md");
    let updated = update(&conn, "LNONE", &builder).expect("Update should not error");

    assert!(!updated, "Update of nonexistent document should return false");
}

#[test]
fn update_batch_changes_multiple_documents() {
    let conn = create_test_db();
    let docs = vec![
        create_task_document("LBT01", "a.md", "a", 2),
        create_task_document("LBT02", "b.md", "b", 2),
        create_task_document("LBT03", "c.md", "c", 2),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let builder = UpdateBuilder::new().priority(Some(1));
    let updated = update_batch(&conn, &["LBT01", "LBT02", "LBT03"], &builder)
        .expect("Batch update should succeed");

    assert_eq!(updated, 3, "Should update 3 documents");

    for id in ["LBT01", "LBT02", "LBT03"] {
        let row = lookup_by_id(&conn, id).expect("Lookup should succeed").unwrap();
        assert_eq!(row.priority, Some(1), "Priority should be updated to 1");
    }
}

#[test]
fn delete_by_id_removes_document() {
    let conn = create_test_db();
    let doc = create_test_document("LDELT", "delete.md", "delete");
    insert(&conn, &doc).expect("Insert should succeed");

    let deleted = delete_by_id(&conn, "LDELT").expect("Delete should succeed");

    assert!(deleted, "Delete should report success");
    assert!(!exists(&conn, "LDELT").expect("Check should succeed"), "Document should not exist");
}

#[test]
fn delete_by_id_nonexistent_returns_false() {
    let conn = create_test_db();

    let deleted = delete_by_id(&conn, "LNONE").expect("Delete should not error");

    assert!(!deleted, "Delete of nonexistent document should return false");
}

#[test]
fn delete_batch_removes_multiple() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LDB01", "a.md", "a"),
        create_test_document("LDB02", "b.md", "b"),
        create_test_document("LDB03", "c.md", "c"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let deleted = delete_batch(&conn, &["LDB01", "LDB02"]).expect("Batch delete should succeed");

    assert_eq!(deleted, 2, "Should delete 2 documents");
    assert!(!exists(&conn, "LDB01").expect("Check should succeed"), "Doc 1 should not exist");
    assert!(!exists(&conn, "LDB02").expect("Check should succeed"), "Doc 2 should not exist");
    assert!(exists(&conn, "LDB03").expect("Check should succeed"), "Doc 3 should still exist");
}

#[test]
fn delete_by_path_prefix_removes_matching() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LPX01", "api/one.md", "one"),
        create_test_document("LPX02", "api/two.md", "two"),
        create_test_document("LPX03", "db/three.md", "three"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let deleted = delete_by_path_prefix(&conn, "api/").expect("Delete should succeed");

    assert_eq!(deleted, 2, "Should delete 2 documents matching prefix");
    assert!(!exists(&conn, "LPX01").expect("Check should succeed"));
    assert!(!exists(&conn, "LPX02").expect("Check should succeed"));
    assert!(exists(&conn, "LPX03").expect("Check should succeed"), "Non-matching should remain");
}

#[test]
fn lookup_by_path_finds_document() {
    let conn = create_test_db();
    let doc = create_test_document("LPATH", "unique/path.md", "path");
    insert(&conn, &doc).expect("Insert should succeed");

    let result = lookup_by_path(&conn, "unique/path.md").expect("Lookup should succeed");

    assert!(result.is_some(), "Document should be found by path");
    assert_eq!(result.unwrap().id, "LPATH");
}

#[test]
fn lookup_by_path_returns_none_for_nonexistent() {
    let conn = create_test_db();

    let result = lookup_by_path(&conn, "nonexistent.md").expect("Lookup should not error");

    assert!(result.is_none(), "Nonexistent path should return None");
}

#[test]
fn lookup_by_name_finds_documents() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LNM01", "a/same.md", "same"),
        create_test_document("LNM02", "b/same.md", "same"),
        create_test_document("LNM03", "c/other.md", "other"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let results = lookup_by_name(&conn, "same").expect("Lookup should succeed");

    assert_eq!(results.len(), 2, "Should find 2 documents with name 'same'");
}

#[test]
fn query_excludes_closed_by_default() {
    let conn = create_test_db();
    let open = create_test_document("LOPEN", "api/tasks/open.md", "open");
    let closed = create_test_document("LCLSD", "api/tasks/.closed/done.md", "done");
    insert(&conn, &open).expect("Insert open should succeed");
    insert(&conn, &closed).expect("Insert closed should succeed");

    let filter = DocumentFilter::new();
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should only find open document");
    assert_eq!(results[0].id, "LOPEN");
}

#[test]
fn query_includes_closed_when_requested() {
    let conn = create_test_db();
    let open = create_test_document("LOPEN", "api/tasks/open.md", "open");
    let closed = create_test_document("LCLSD", "api/tasks/.closed/done.md", "done");
    insert(&conn, &open).expect("Insert open should succeed");
    insert(&conn, &closed).expect("Insert closed should succeed");

    let filter = DocumentFilter::including_closed();
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should find both open and closed documents");
}

#[test]
fn query_filters_by_priority() {
    let conn = create_test_db();
    let docs = vec![
        create_task_document("LP0", "p0.md", "p0", 0),
        create_task_document("LP1", "p1.md", "p1", 1),
        create_task_document("LP2", "p2.md", "p2", 2),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_priority(1);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find one document with priority 1");
    assert_eq!(results[0].id, "LP1");
}

#[test]
fn query_filters_by_priority_range() {
    let conn = create_test_db();
    let docs = vec![
        create_task_document("LR0", "r0.md", "r0", 0),
        create_task_document("LR1", "r1.md", "r1", 1),
        create_task_document("LR2", "r2.md", "r2", 2),
        create_task_document("LR3", "r3.md", "r3", 3),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_priority_range(1, 2);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should find documents with priority 1-2");
}

#[test]
fn query_filters_by_task_type() {
    let conn = create_test_db();
    insert(
        &conn,
        &InsertDocument::new(
            "LBUG1".to_string(),
            None,
            "bug.md".to_string(),
            "bug".to_string(),
            "A bug".to_string(),
            Some(TaskType::Bug),
            Some(2),
            None,
            None,
            None,
            "hash".to_string(),
            100,
            false,
        ),
    )
    .expect("Insert bug should succeed");
    insert(
        &conn,
        &InsertDocument::new(
            "LFEAT".to_string(),
            None,
            "feat.md".to_string(),
            "feat".to_string(),
            "A feature".to_string(),
            Some(TaskType::Feature),
            Some(2),
            None,
            None,
            None,
            "hash".to_string(),
            100,
            false,
        ),
    )
    .expect("Insert feature should succeed");

    let filter = DocumentFilter::including_closed().with_task_type(TaskType::Bug);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find one bug");
    assert_eq!(results[0].id, "LBUG1");
}

#[test]
fn query_filters_by_path_prefix() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LAPI1", "api/one.md", "one"),
        create_test_document("LAPI2", "api/two.md", "two"),
        create_test_document("LDBS1", "db/schema.md", "schema"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_path_prefix("api/");
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should find documents under api/");
}

#[test]
fn query_filters_by_is_root() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LROOT", "api/api.md", "api"),
        create_test_document("LCHLD", "api/child.md", "child"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_is_root(true);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find one root document");
    assert_eq!(results[0].id, "LROOT");
}

#[test]
fn query_filters_by_in_tasks_dir() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LTASK", "api/tasks/fix.md", "fix"),
        create_test_document("LDOCS", "api/docs/design.md", "design"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_in_tasks_dir(true);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find one task");
    assert_eq!(results[0].id, "LTASK");
}

#[test]
fn query_sorts_by_name_ascending() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LC", "c.md", "charlie"),
        create_test_document("LA", "a.md", "alpha"),
        create_test_document("LB", "b.md", "bravo"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed()
        .sort_by(SortColumn::Name)
        .sort_order(SortOrder::Ascending);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results[0].name, "alpha");
    assert_eq!(results[1].name, "bravo");
    assert_eq!(results[2].name, "charlie");
}

#[test]
fn query_limits_results() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("L1", "1.md", "one"),
        create_test_document("L2", "2.md", "two"),
        create_test_document("L3", "3.md", "three"),
        create_test_document("L4", "4.md", "four"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().limit(2);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should limit to 2 results");
}

#[test]
fn query_offsets_results() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("L1", "1.md", "aaa"),
        create_test_document("L2", "2.md", "bbb"),
        create_test_document("L3", "3.md", "ccc"),
        create_test_document("L4", "4.md", "ddd"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed()
        .sort_by(SortColumn::Name)
        .sort_order(SortOrder::Ascending)
        .limit(2)
        .offset(1);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should return 2 results");
    assert_eq!(results[0].name, "bbb", "First result should be offset by 1");
    assert_eq!(results[1].name, "ccc");
}

#[test]
fn count_returns_correct_count() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("L1", "1.md", "one"),
        create_test_document("L2", "2.md", "two"),
        create_test_document("L3", "3.md", "three"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed();
    let result = count(&conn, &filter).expect("Count should succeed");

    assert_eq!(result, 3, "Should count 3 documents");
}

#[test]
fn count_respects_filters() {
    let conn = create_test_db();
    let docs = vec![
        create_task_document("LT1", "t1.md", "t1", 0),
        create_task_document("LT2", "t2.md", "t2", 1),
        create_task_document("LT3", "t3.md", "t3", 2),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let filter = DocumentFilter::including_closed().with_priority_range(0, 1);
    let result = count(&conn, &filter).expect("Count should succeed");

    assert_eq!(result, 2, "Should count 2 documents with priority 0-1");
}

#[test]
fn exists_returns_true_for_existing_document() {
    let conn = create_test_db();
    let doc = create_test_document("LEXST", "exists.md", "exists");
    insert(&conn, &doc).expect("Insert should succeed");

    assert!(exists(&conn, "LEXST").expect("Check should succeed"));
}

#[test]
fn exists_returns_false_for_nonexistent_document() {
    let conn = create_test_db();

    assert!(!exists(&conn, "LNONE").expect("Check should succeed"));
}

#[test]
fn exists_at_path_returns_true_for_existing_path() {
    let conn = create_test_db();
    let doc = create_test_document("LPEXS", "path/exists.md", "exists");
    insert(&conn, &doc).expect("Insert should succeed");

    assert!(exists_at_path(&conn, "path/exists.md").expect("Check should succeed"));
}

#[test]
fn exists_at_path_returns_false_for_nonexistent_path() {
    let conn = create_test_db();

    assert!(!exists_at_path(&conn, "nonexistent.md").expect("Check should succeed"));
}

#[test]
fn all_ids_returns_all_document_ids() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LAID1", "a.md", "a"),
        create_test_document("LAID2", "b.md", "b"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let ids = all_ids(&conn).expect("Should get all IDs");

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"LAID1".to_string()));
    assert!(ids.contains(&"LAID2".to_string()));
}

#[test]
fn all_paths_returns_all_document_paths() {
    let conn = create_test_db();
    let docs = vec![
        create_test_document("LAP01", "path/a.md", "a"),
        create_test_document("LAP02", "path/b.md", "b"),
    ];
    insert_batch(&conn, &docs).expect("Batch insert should succeed");

    let paths = all_paths(&conn).expect("Should get all paths");

    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"path/a.md".to_string()));
    assert!(paths.contains(&"path/b.md".to_string()));
}

#[test]
fn insert_document_preserves_task_type() {
    let conn = create_test_db();
    let doc = InsertDocument::new(
        "LTYP1".to_string(),
        None,
        "bug.md".to_string(),
        "bug".to_string(),
        "A bug".to_string(),
        Some(TaskType::Bug),
        Some(1),
        None,
        None,
        None,
        "hash".to_string(),
        100,
        false,
    );

    insert(&conn, &doc).expect("Insert should succeed");

    let row = lookup_by_id(&conn, "LTYP1").expect("Lookup should succeed").unwrap();
    assert_eq!(row.task_type, Some(TaskType::Bug));
}

#[test]
fn document_filter_builder_pattern_works() {
    let filter = DocumentFilter::new()
        .with_priority(1)
        .with_task_type(TaskType::Bug)
        .with_path_prefix("api/")
        .with_is_root(false)
        .sort_by(SortColumn::Priority)
        .sort_order(SortOrder::Ascending)
        .limit(10)
        .offset(5);

    assert_eq!(filter.priority, Some(1));
    assert_eq!(filter.task_type, Some(TaskType::Bug));
    assert_eq!(filter.path_prefix, Some("api/".to_string()));
    assert_eq!(filter.is_root, Some(false));
    assert_eq!(filter.limit, Some(10));
    assert_eq!(filter.offset, Some(5));
}

#[test]
fn query_filters_by_state_closed() {
    let conn = create_test_db();
    let open = create_test_document("LOPEN", "api/tasks/open.md", "open");
    let closed = create_test_document("LCLSD", "api/tasks/.closed/done.md", "done");
    insert(&conn, &open).expect("Insert open should succeed");
    insert(&conn, &closed).expect("Insert closed should succeed");

    let filter = DocumentFilter::including_closed().with_state(DocumentState::Closed);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find only closed document");
    assert_eq!(results[0].id, "LCLSD");
}

#[test]
fn query_filters_by_state_open() {
    let conn = create_test_db();
    let open = create_test_document("LOPEN", "api/tasks/open.md", "open");
    let closed = create_test_document("LCLSD", "api/tasks/.closed/done.md", "done");
    insert(&conn, &open).expect("Insert open should succeed");
    insert(&conn, &closed).expect("Insert closed should succeed");

    let filter = DocumentFilter::including_closed().with_state(DocumentState::Open);
    let results = query(&conn, &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should find only open document");
    assert_eq!(results[0].id, "LOPEN");
}

#[test]
fn query_filters_by_state_blocked_executes_without_error() {
    // This test verifies that the blocked state query executes successfully.
    // The blocked state filter depends on links being stored with link_type =
    // 'blocked_by', which is implemented in the link_queries module. Until that
    // module is complete, this query will return empty results, but we verify
    // it doesn't error.
    let conn = create_test_db();
    let doc = create_test_document("LTEST", "api/tasks/test.md", "test");
    insert(&conn, &doc).expect("Insert should succeed");

    let filter = DocumentFilter::including_closed().with_state(DocumentState::Blocked);
    let results = query(&conn, &filter).expect("Blocked state query should execute without error");

    // Without blocked_by links stored, no documents will be returned
    assert!(results.is_empty(), "Without blocked_by links, no documents should be blocked");
}
