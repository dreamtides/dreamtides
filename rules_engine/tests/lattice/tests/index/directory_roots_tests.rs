use lattice::index::directory_roots::{
    DirectoryRoot, clear_all, delete, get, get_ancestors, get_children, get_root_id, list_all,
    list_at_depth, upsert,
};
use lattice::index::schema_definition;
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn make_root(path: &str, id: &str, parent: Option<&str>, depth: u32) -> DirectoryRoot {
    DirectoryRoot {
        directory_path: path.to_string(),
        root_id: id.to_string(),
        parent_path: parent.map(|s| s.to_string()),
        depth,
    }
}

// upsert tests

#[test]
fn upsert_inserts_new_entry() {
    let conn = create_test_db();
    let root = make_root("api", "L001", None, 0);

    upsert(&conn, &root).expect("Upsert should succeed");

    let result = get(&conn, "api").expect("Get should succeed");
    assert!(result.is_some(), "Entry should exist after upsert");
    let entry = result.unwrap();
    assert_eq!(entry.root_id, "L001", "Root ID should match");
    assert_eq!(entry.depth, 0, "Depth should match");
}

#[test]
fn upsert_updates_existing_entry() {
    let conn = create_test_db();
    let root1 = make_root("api", "L001", None, 0);
    let root2 = make_root("api", "L002", Some("parent"), 1);

    upsert(&conn, &root1).expect("First upsert should succeed");
    upsert(&conn, &root2).expect("Second upsert should succeed");

    let result = get(&conn, "api").expect("Get should succeed").unwrap();
    assert_eq!(result.root_id, "L002", "Root ID should be updated");
    assert_eq!(result.parent_path, Some("parent".to_string()), "Parent should be updated");
    assert_eq!(result.depth, 1, "Depth should be updated");
}

#[test]
fn upsert_handles_entry_with_parent() {
    let conn = create_test_db();
    let root = make_root("api/users", "L003", Some("api"), 1);

    upsert(&conn, &root).expect("Upsert should succeed");

    let result = get(&conn, "api/users").expect("Get should succeed").unwrap();
    assert_eq!(result.parent_path, Some("api".to_string()));
}

// get tests

#[test]
fn get_returns_none_for_missing_entry() {
    let conn = create_test_db();

    let result = get(&conn, "nonexistent").expect("Get should succeed");

    assert!(result.is_none(), "Should return None for non-existent path");
}

#[test]
fn get_returns_full_entry() {
    let conn = create_test_db();
    let root = make_root("database", "L004", Some("parent"), 2);
    upsert(&conn, &root).expect("Upsert should succeed");

    let result = get(&conn, "database").expect("Get should succeed").unwrap();

    assert_eq!(result.directory_path, "database");
    assert_eq!(result.root_id, "L004");
    assert_eq!(result.parent_path, Some("parent".to_string()));
    assert_eq!(result.depth, 2);
}

// get_root_id tests

#[test]
fn get_root_id_returns_none_for_missing() {
    let conn = create_test_db();

    let result = get_root_id(&conn, "missing").expect("Should succeed");

    assert!(result.is_none(), "Should return None for non-existent directory");
}

#[test]
fn get_root_id_returns_id_only() {
    let conn = create_test_db();
    let root = make_root("services", "L005", None, 0);
    upsert(&conn, &root).expect("Upsert should succeed");

    let result = get_root_id(&conn, "services").expect("Should succeed");

    assert_eq!(result, Some("L005".to_string()), "Should return just the root ID");
}

// get_ancestors tests

#[test]
fn get_ancestors_returns_empty_for_missing() {
    let conn = create_test_db();

    let result = get_ancestors(&conn, "nonexistent").expect("Should succeed");

    assert!(result.is_empty(), "Should return empty vec for non-existent path");
}

#[test]
fn get_ancestors_returns_single_root() {
    let conn = create_test_db();
    let root = make_root("api", "L010", None, 0);
    upsert(&conn, &root).expect("Upsert should succeed");

    let result = get_ancestors(&conn, "api").expect("Should succeed");

    assert_eq!(result.len(), 1, "Should return single entry for root with no parent");
    assert_eq!(result[0].directory_path, "api");
}

#[test]
fn get_ancestors_returns_chain_root_first() {
    let conn = create_test_db();
    // Create a three-level hierarchy: root -> api -> api/users
    upsert(&conn, &make_root("root", "L011", None, 0)).unwrap();
    upsert(&conn, &make_root("api", "L012", Some("root"), 1)).unwrap();
    upsert(&conn, &make_root("api/users", "L013", Some("api"), 2)).unwrap();

    let result = get_ancestors(&conn, "api/users").expect("Should succeed");

    assert_eq!(result.len(), 3, "Should return all three levels");
    assert_eq!(result[0].directory_path, "root", "First should be root");
    assert_eq!(result[1].directory_path, "api", "Second should be api");
    assert_eq!(result[2].directory_path, "api/users", "Third should be api/users");
}

#[test]
fn get_ancestors_stops_at_broken_chain() {
    let conn = create_test_db();
    // api/users has parent "api" but api doesn't exist
    upsert(&conn, &make_root("api/users", "L014", Some("api"), 1)).unwrap();

    let result = get_ancestors(&conn, "api/users").expect("Should succeed");

    assert_eq!(result.len(), 1, "Should only return existing entry when parent is missing");
}

// get_children tests

#[test]
fn get_children_returns_empty_when_no_children() {
    let conn = create_test_db();
    upsert(&conn, &make_root("parent", "L020", None, 0)).unwrap();

    let result = get_children(&conn, "parent").expect("Should succeed");

    assert!(result.is_empty(), "Should return empty vec when no children exist");
}

#[test]
fn get_children_returns_immediate_children_only() {
    let conn = create_test_db();
    upsert(&conn, &make_root("root", "L021", None, 0)).unwrap();
    upsert(&conn, &make_root("api", "L022", Some("root"), 1)).unwrap();
    upsert(&conn, &make_root("database", "L023", Some("root"), 1)).unwrap();
    upsert(&conn, &make_root("api/users", "L024", Some("api"), 2)).unwrap();

    let result = get_children(&conn, "root").expect("Should succeed");

    assert_eq!(result.len(), 2, "Should return only immediate children");
    let paths: Vec<_> = result.iter().map(|r| r.directory_path.as_str()).collect();
    assert!(paths.contains(&"api"), "Should contain api");
    assert!(paths.contains(&"database"), "Should contain database");
    assert!(!paths.contains(&"api/users"), "Should not contain grandchild");
}

#[test]
fn get_children_returns_sorted_by_path() {
    let conn = create_test_db();
    upsert(&conn, &make_root("parent", "L025", None, 0)).unwrap();
    upsert(&conn, &make_root("z-child", "L026", Some("parent"), 1)).unwrap();
    upsert(&conn, &make_root("a-child", "L027", Some("parent"), 1)).unwrap();
    upsert(&conn, &make_root("m-child", "L028", Some("parent"), 1)).unwrap();

    let result = get_children(&conn, "parent").expect("Should succeed");

    assert_eq!(result[0].directory_path, "a-child", "First should be a-child");
    assert_eq!(result[1].directory_path, "m-child", "Second should be m-child");
    assert_eq!(result[2].directory_path, "z-child", "Third should be z-child");
}

// list_at_depth tests

#[test]
fn list_at_depth_returns_empty_when_none_at_depth() {
    let conn = create_test_db();
    upsert(&conn, &make_root("root", "L030", None, 0)).unwrap();

    let result = list_at_depth(&conn, 5).expect("Should succeed");

    assert!(result.is_empty(), "Should return empty vec when no roots at depth");
}

#[test]
fn list_at_depth_returns_all_at_specified_depth() {
    let conn = create_test_db();
    upsert(&conn, &make_root("root1", "L031", None, 0)).unwrap();
    upsert(&conn, &make_root("root2", "L032", None, 0)).unwrap();
    upsert(&conn, &make_root("child1", "L033", Some("root1"), 1)).unwrap();
    upsert(&conn, &make_root("child2", "L034", Some("root2"), 1)).unwrap();

    let depth_0 = list_at_depth(&conn, 0).expect("Should succeed");
    let depth_1 = list_at_depth(&conn, 1).expect("Should succeed");

    assert_eq!(depth_0.len(), 2, "Should return 2 roots at depth 0");
    assert_eq!(depth_1.len(), 2, "Should return 2 roots at depth 1");
}

#[test]
fn list_at_depth_returns_sorted_by_path() {
    let conn = create_test_db();
    upsert(&conn, &make_root("zebra", "L035", None, 0)).unwrap();
    upsert(&conn, &make_root("apple", "L036", None, 0)).unwrap();

    let result = list_at_depth(&conn, 0).expect("Should succeed");

    assert_eq!(result[0].directory_path, "apple", "First should be apple");
    assert_eq!(result[1].directory_path, "zebra", "Second should be zebra");
}

// list_all tests

#[test]
fn list_all_returns_empty_initially() {
    let conn = create_test_db();

    let result = list_all(&conn).expect("Should succeed");

    assert!(result.is_empty(), "Should return empty vec when no entries");
}

#[test]
fn list_all_returns_ordered_by_depth_then_path() {
    let conn = create_test_db();
    upsert(&conn, &make_root("z-root", "L040", None, 0)).unwrap();
    upsert(&conn, &make_root("a-root", "L041", None, 0)).unwrap();
    upsert(&conn, &make_root("z-child", "L042", Some("z-root"), 1)).unwrap();
    upsert(&conn, &make_root("a-child", "L043", Some("a-root"), 1)).unwrap();
    upsert(&conn, &make_root("deep", "L044", Some("a-child"), 2)).unwrap();

    let result = list_all(&conn).expect("Should succeed");

    assert_eq!(result.len(), 5, "Should return all entries");
    // Depth 0 first, sorted by path
    assert_eq!(result[0].directory_path, "a-root", "First should be a-root (depth 0)");
    assert_eq!(result[1].directory_path, "z-root", "Second should be z-root (depth 0)");
    // Depth 1 next, sorted by path
    assert_eq!(result[2].directory_path, "a-child", "Third should be a-child (depth 1)");
    assert_eq!(result[3].directory_path, "z-child", "Fourth should be z-child (depth 1)");
    // Depth 2 last
    assert_eq!(result[4].directory_path, "deep", "Fifth should be deep (depth 2)");
}

// delete tests

#[test]
fn delete_returns_false_for_missing() {
    let conn = create_test_db();

    let deleted = delete(&conn, "nonexistent").expect("Delete should succeed");

    assert!(!deleted, "Delete should return false for non-existent path");
}

#[test]
fn delete_removes_existing_entry() {
    let conn = create_test_db();
    upsert(&conn, &make_root("to-delete", "L050", None, 0)).unwrap();

    let deleted = delete(&conn, "to-delete").expect("Delete should succeed");

    assert!(deleted, "Delete should return true for existing entry");
    let result = get(&conn, "to-delete").expect("Get should succeed");
    assert!(result.is_none(), "Entry should no longer exist after delete");
}

#[test]
fn delete_does_not_affect_other_entries() {
    let conn = create_test_db();
    upsert(&conn, &make_root("keep", "L051", None, 0)).unwrap();
    upsert(&conn, &make_root("remove", "L052", None, 0)).unwrap();

    delete(&conn, "remove").expect("Delete should succeed");

    let remaining = get(&conn, "keep").expect("Get should succeed");
    assert!(remaining.is_some(), "Other entry should not be affected");
}

// clear_all tests

#[test]
fn clear_all_returns_zero_when_empty() {
    let conn = create_test_db();

    let count = clear_all(&conn).expect("Clear all should succeed");

    assert_eq!(count, 0, "Should return 0 when no entries existed");
}

#[test]
fn clear_all_removes_all_entries() {
    let conn = create_test_db();
    upsert(&conn, &make_root("root1", "L060", None, 0)).unwrap();
    upsert(&conn, &make_root("root2", "L061", None, 0)).unwrap();
    upsert(&conn, &make_root("child", "L062", Some("root1"), 1)).unwrap();

    let count = clear_all(&conn).expect("Clear all should succeed");

    assert_eq!(count, 3, "Should return count of deleted entries");
    let remaining = list_all(&conn).expect("List all should succeed");
    assert!(remaining.is_empty(), "No entries should remain after clear_all");
}

#[test]
fn clear_all_allows_reinsertion() {
    let conn = create_test_db();
    upsert(&conn, &make_root("test", "L070", None, 0)).unwrap();
    clear_all(&conn).expect("Clear all should succeed");

    // Should be able to insert again with same path
    upsert(&conn, &make_root("test", "L071", None, 0)).expect("Reinsertion should succeed");

    let result = get(&conn, "test").expect("Get should succeed").unwrap();
    assert_eq!(result.root_id, "L071", "New entry should have new ID");
}
