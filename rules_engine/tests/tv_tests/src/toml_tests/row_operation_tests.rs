use std::collections::HashMap;

use serde_json::json;
use tv_lib::error::error_types::TvError;

use crate::test_utils::harness::TvTestHarness;

#[test]
fn test_add_row_at_end() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "add_end.toml",
        r#"[[cards]]
id = "card-1"
name = "First"

[[cards]]
id = "card-2"
name = "Second"
"#,
    );

    // Add row at end (no position specified)
    let result = harness.add_row(&path, "cards", None, None);
    assert!(result.is_ok(), "Add row should succeed: {:?}", result);

    let add_result = result.unwrap();
    assert!(add_result.success);
    assert_eq!(add_result.row_index, 2, "New row should be at index 2");

    // Verify the table now has 3 rows
    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 3, "Table should have 3 rows");

    // Verify file content has new [[cards]] entry
    let content = harness.read_file_content(&path);
    assert_eq!(content.matches("[[cards]]").count(), 3, "File should have 3 card entries");
}

#[test]
fn test_add_row_at_position() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "add_position.toml",
        r#"[[cards]]
id = "card-1"
name = "First"

[[cards]]
id = "card-3"
name = "Third"
"#,
    );

    // Add row at position 1 (between first and second)
    let mut initial_values = HashMap::new();
    initial_values.insert("id".to_string(), json!("card-2"));
    initial_values.insert("name".to_string(), json!("Second"));

    let result = harness.add_row(&path, "cards", Some(1), Some(initial_values));
    assert!(result.is_ok(), "Add row should succeed: {:?}", result);

    let add_result = result.unwrap();
    assert!(add_result.success);
    assert_eq!(add_result.row_index, 1, "New row should be at index 1");

    // Verify ordering is preserved
    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 3);

    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-1"));
    assert_eq!(table.rows[1][id_idx].as_str(), Some("card-2"));
    assert_eq!(table.rows[2][id_idx].as_str(), Some("card-3"));
}

#[test]
fn test_add_row_at_beginning() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "add_beginning.toml",
        r#"[[cards]]
id = "card-2"
name = "Second"
"#,
    );

    let mut initial_values = HashMap::new();
    initial_values.insert("id".to_string(), json!("card-1"));
    initial_values.insert("name".to_string(), json!("First"));

    let result = harness.add_row(&path, "cards", Some(0), Some(initial_values));
    assert!(result.is_ok(), "Add row should succeed: {:?}", result);

    let table = harness.load_table(&path, "cards").unwrap();
    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();

    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-1"));
    assert_eq!(table.rows[1][id_idx].as_str(), Some("card-2"));
}

#[test]
fn test_add_row_with_initial_values() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "add_values.toml",
        r#"[[cards]]
id = "card-1"
name = "First"
cost = 1
"#,
    );

    let mut initial_values = HashMap::new();
    initial_values.insert("id".to_string(), json!("card-2"));
    initial_values.insert("name".to_string(), json!("Second"));
    initial_values.insert("cost".to_string(), json!(5));
    initial_values.insert("rarity".to_string(), json!("rare"));

    let result = harness.add_row(&path, "cards", None, Some(initial_values));
    assert!(result.is_ok());

    let table = harness.load_table(&path, "cards").unwrap();
    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    let cost_idx = table.headers.iter().position(|h| h == "cost").unwrap();
    let rarity_idx = table.headers.iter().position(|h| h == "rarity").unwrap();

    assert_eq!(table.rows[1][id_idx].as_str(), Some("card-2"));
    assert_eq!(table.rows[1][cost_idx].as_i64(), Some(5));
    assert_eq!(table.rows[1][rarity_idx].as_str(), Some("rare"));
}

#[test]
fn test_add_row_position_out_of_bounds() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bounds.toml",
        r#"[[cards]]
id = "card-1"
"#,
    );

    // Try to insert at position 5 when array has 1 element
    let result = harness.add_row(&path, "cards", Some(5), None);

    match result {
        Err(TvError::RowNotFound { table_name, row_index }) => {
            assert_eq!(table_name, "cards");
            assert_eq!(row_index, 5);
        }
        other => panic!("Expected RowNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_add_row_to_empty_array() {
    // Note: TOML doesn't support empty arrays of tables. When all rows are deleted,
    // the [[table]] entry is removed entirely. Adding a row to a non-existent table
    // is not supported. This test verifies the expected behavior: attempting to add
    // to a table that was deleted returns TableNotFound error.
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "empty_start.toml",
        r#"[[cards]]
id = "temp"
"#,
    );

    // Delete the only row - this removes the [[cards]] table entirely
    harness.delete_row(&path, "cards", 0).unwrap();

    // Trying to add a row to the now-nonexistent table should fail
    let mut initial_values = HashMap::new();
    initial_values.insert("id".to_string(), json!("card-1"));

    let result = harness.add_row(&path, "cards", None, Some(initial_values));

    // This is expected to fail since the table no longer exists
    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_add_row_table_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "wrong_table.toml",
        r#"[[items]]
id = "item-1"
"#,
    );

    let result = harness.add_row(&path, "cards", None, None);

    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_delete_row() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "delete.toml",
        r#"[[cards]]
id = "card-1"
name = "First"

[[cards]]
id = "card-2"
name = "Second"

[[cards]]
id = "card-3"
name = "Third"
"#,
    );

    // Delete the middle row
    let result = harness.delete_row(&path, "cards", 1);
    assert!(result.is_ok(), "Delete row should succeed: {:?}", result);

    let delete_result = result.unwrap();
    assert!(delete_result.success);
    assert_eq!(delete_result.deleted_index, 1);

    // Verify the table now has 2 rows
    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 2);

    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-1"));
    assert_eq!(table.rows[1][id_idx].as_str(), Some("card-3"));
}

#[test]
fn test_delete_first_row() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "delete_first.toml",
        r#"[[cards]]
id = "card-1"

[[cards]]
id = "card-2"
"#,
    );

    let result = harness.delete_row(&path, "cards", 0);
    assert!(result.is_ok());

    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 1);

    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-2"));
}

#[test]
fn test_delete_last_row() {
    // Note: When the last row is deleted from a TOML array of tables, the entire
    // [[table]] section is removed because TOML doesn't support empty arrays of
    // tables. This is expected behavior.
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "delete_last.toml",
        r#"[[cards]]
id = "only-card"
name = "The Only One"
"#,
    );

    // Delete the only row
    let result = harness.delete_row(&path, "cards", 0);
    assert!(result.is_ok(), "Delete row should succeed: {:?}", result);

    // After deleting the last row, the table no longer exists
    // Attempting to load it returns TableNotFound
    let load_result = harness.load_table(&path, "cards");
    match load_result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error after deleting last row, got: {other:?}"),
    }

    // Verify file content no longer has the card data
    let content = harness.read_file_content(&path);
    assert!(!content.contains("only-card"), "Deleted card ID should not be in file");
    assert!(!content.contains("[[cards]]"), "Table header should be removed");
}

#[test]
fn test_delete_row_out_of_bounds() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bounds.toml",
        r#"[[cards]]
id = "card-1"
"#,
    );

    let result = harness.delete_row(&path, "cards", 5);

    match result {
        Err(TvError::RowNotFound { table_name, row_index }) => {
            assert_eq!(table_name, "cards");
            assert_eq!(row_index, 5);
        }
        other => panic!("Expected RowNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_delete_row_table_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "wrong.toml",
        r#"[[items]]
id = "item-1"
"#,
    );

    let result = harness.delete_row(&path, "cards", 0);

    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_delete_row_preserves_comments() {
    let harness = TvTestHarness::new();
    let original = r#"# Header comment

[[cards]]
# First card
id = "card-1"

[[cards]]
# Second card - to be deleted
id = "card-2"

[[cards]]
# Third card
id = "card-3"
"#;
    let path = harness.create_toml_file("comments.toml", original);

    harness.delete_row(&path, "cards", 1).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("# Header comment"), "Header comment should be preserved");
    assert!(content.contains("# First card"), "First card comment should be preserved");
    assert!(content.contains("# Third card"), "Third card comment should be preserved");
    assert!(!content.contains("card-2"), "Deleted card should not be present");
}

#[test]
fn test_add_row_preserves_comments() {
    let harness = TvTestHarness::new();
    let original = r#"# Header comment

[[cards]]
# First card
id = "card-1"
"#;
    let path = harness.create_toml_file("add_comments.toml", original);

    let mut initial_values = HashMap::new();
    initial_values.insert("id".to_string(), json!("card-2"));

    harness.add_row(&path, "cards", None, Some(initial_values)).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("# Header comment"), "Header comment should be preserved");
    assert!(content.contains("# First card"), "First card comment should be preserved");
    assert!(content.contains("card-2"), "New card should be added");
}

#[test]
fn test_batch_paste_multiple_cells() {
    // This test verifies that paste operations (multiple cells) work in a single
    // atomic write This is already covered by save_tests.rs but included here
    // for completeness as it's mentioned in Task 23 requirements
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "paste.toml",
        r#"[[cards]]
a = ""
b = ""
c = ""

[[cards]]
a = ""
b = ""
c = ""
"#,
    );

    use tv_lib::toml::document_writer::CellUpdate;

    // Simulate pasting a 2x3 block of data
    let updates = vec![
        CellUpdate { row_index: 0, column_key: "a".to_string(), value: json!("A1") },
        CellUpdate { row_index: 0, column_key: "b".to_string(), value: json!("B1") },
        CellUpdate { row_index: 0, column_key: "c".to_string(), value: json!("C1") },
        CellUpdate { row_index: 1, column_key: "a".to_string(), value: json!("A2") },
        CellUpdate { row_index: 1, column_key: "b".to_string(), value: json!("B2") },
        CellUpdate { row_index: 1, column_key: "c".to_string(), value: json!("C2") },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok(), "Batch paste should succeed: {result:?}");

    let batch_result = result.unwrap();
    assert!(batch_result.success);
    assert_eq!(batch_result.applied_count, 6, "All 6 cells should be updated");

    let table = harness.load_table(&path, "cards").unwrap();
    let a_idx = table.headers.iter().position(|h| h == "a").unwrap();
    let b_idx = table.headers.iter().position(|h| h == "b").unwrap();
    let c_idx = table.headers.iter().position(|h| h == "c").unwrap();

    assert_eq!(table.rows[0][a_idx].as_str(), Some("A1"));
    assert_eq!(table.rows[0][b_idx].as_str(), Some("B1"));
    assert_eq!(table.rows[0][c_idx].as_str(), Some("C1"));
    assert_eq!(table.rows[1][a_idx].as_str(), Some("A2"));
    assert_eq!(table.rows[1][b_idx].as_str(), Some("B2"));
    assert_eq!(table.rows[1][c_idx].as_str(), Some("C2"));
}

#[test]
fn test_add_delete_roundtrip() {
    // Test that adding and then deleting restores original state
    let harness = TvTestHarness::new();
    let original = r#"[[cards]]
id = "card-1"
name = "First"
"#;
    let path = harness.create_toml_file("roundtrip.toml", original);

    // Add a row
    let mut values = HashMap::new();
    values.insert("id".to_string(), json!("card-2"));
    values.insert("name".to_string(), json!("Second"));
    harness.add_row(&path, "cards", None, Some(values)).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 2);

    // Delete the added row
    harness.delete_row(&path, "cards", 1).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 1);

    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-1"));
}

#[test]
fn test_multiple_row_operations() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "multi_ops.toml",
        r#"[[cards]]
id = "card-1"
"#,
    );

    // Add multiple rows
    for i in 2..=5 {
        let mut values = HashMap::new();
        values.insert("id".to_string(), json!(format!("card-{}", i)));
        harness.add_row(&path, "cards", None, Some(values)).unwrap();
    }

    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 5);

    // Delete alternating rows (indices 1, 2 after first deletion)
    harness.delete_row(&path, "cards", 1).unwrap(); // removes card-2
    harness.delete_row(&path, "cards", 2).unwrap(); // removes card-4

    let table = harness.load_table(&path, "cards").unwrap();
    assert_eq!(table.rows.len(), 3);

    let id_idx = table.headers.iter().position(|h| h == "id").unwrap();
    assert_eq!(table.rows[0][id_idx].as_str(), Some("card-1"));
    assert_eq!(table.rows[1][id_idx].as_str(), Some("card-3"));
    assert_eq!(table.rows[2][id_idx].as_str(), Some("card-5"));
}
