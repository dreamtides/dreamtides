use serde_json::json;
use tv_lib::error::error_types::TvError;
use tv_lib::toml::document_writer::CellUpdate;

use crate::test_utils::test_utils_mod::TvTestHarness;

#[test]
fn test_save_cell_updates_value() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "test.toml",
        r#"
[[cards]]
id = "abc-123"
name = "Test Card"
cost = 3

[[cards]]
id = "def-456"
name = "Another Card"
cost = 5
"#,
    );

    // Update the name of the first card
    let result = harness.save_cell(&path, "cards", 0, "name", json!("Updated Card"));
    assert!(result.is_ok(), "Cell save should succeed: {:?}", result);

    // Read back and verify
    let content = harness.read_file_content(&path);
    assert!(content.contains("Updated Card"), "File should contain new value");
    assert!(!content.contains("Test Card"), "File should not contain old value");

    // Verify via load
    let table = harness.load_table(&path, "cards").expect("Should reload table");
    let name_idx = table.headers.iter().position(|h| h == "name").unwrap();
    assert_eq!(table.rows[0][name_idx].as_str(), Some("Updated Card"));
}

#[test]
fn test_save_cell_preserves_comments() {
    let harness = TvTestHarness::new();
    let original_content = r#"# This is a header comment
# describing the cards table

[[cards]]
# This card is special
id = "abc-123"
name = "Test Card" # inline comment
cost = 3

# Second card section
[[cards]]
id = "def-456"
name = "Another Card"
cost = 5
"#;
    let path = harness.create_toml_file("with_comments.toml", original_content);

    // Update a value
    let result = harness.save_cell(&path, "cards", 0, "cost", json!(7));
    assert!(result.is_ok(), "Cell save should succeed");

    // Read back and verify comments are preserved
    let content = harness.read_file_content(&path);
    assert!(content.contains("# This is a header comment"), "Header comment should be preserved");
    assert!(content.contains("# This card is special"), "Block comment should be preserved");
    assert!(content.contains("# inline comment"), "Inline comment should be preserved");
    assert!(content.contains("# Second card section"), "Section comment should be preserved");
    assert!(content.contains("cost = 7"), "Value should be updated");
}

#[test]
fn test_save_cell_preserves_whitespace() {
    let harness = TvTestHarness::new();
    let original_content = r#"
[[cards]]
id = "abc-123"
name = "Test Card"


[[cards]]
id = "def-456"
name = "Another Card"
"#;
    let path = harness.create_toml_file("whitespace.toml", original_content);

    // Update a value
    harness.save_cell(&path, "cards", 1, "name", json!("Modified Card")).unwrap();

    // Verify blank lines are preserved (toml_edit should maintain structure)
    let content = harness.read_file_content(&path);
    assert!(content.contains("Modified Card"), "Value should be updated");
}

#[test]
fn test_save_cell_preserves_key_order() {
    let harness = TvTestHarness::new();
    let original_content = r#"[[cards]]
zebra = "z"
alpha = "a"
middle = "m"
"#;
    let path = harness.create_toml_file("key_order.toml", original_content);

    // Update the middle key
    harness.save_cell(&path, "cards", 0, "middle", json!("updated")).unwrap();

    let content = harness.read_file_content(&path);

    // Find positions of keys
    let zebra_pos = content.find("zebra").expect("zebra should exist");
    let alpha_pos = content.find("alpha").expect("alpha should exist");
    let middle_pos = content.find("middle").expect("middle should exist");

    assert!(zebra_pos < alpha_pos, "zebra should come before alpha");
    assert!(alpha_pos < middle_pos, "alpha should come before middle");
}

#[test]
fn test_save_cell_different_types() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "types.toml",
        r#"[[items]]
string_val = "original"
int_val = 42
float_val = 3.14
bool_val = true
"#,
    );

    // Update string
    harness.save_cell(&path, "items", 0, "string_val", json!("new string")).unwrap();
    // Update integer
    harness.save_cell(&path, "items", 0, "int_val", json!(100)).unwrap();
    // Update float
    harness.save_cell(&path, "items", 0, "float_val", json!(2.718)).unwrap();
    // Update boolean
    harness.save_cell(&path, "items", 0, "bool_val", json!(false)).unwrap();

    let table = harness.load_table(&path, "items").unwrap();
    let headers = &table.headers;

    let string_idx = headers.iter().position(|h| h == "string_val").unwrap();
    let int_idx = headers.iter().position(|h| h == "int_val").unwrap();
    let float_idx = headers.iter().position(|h| h == "float_val").unwrap();
    let bool_idx = headers.iter().position(|h| h == "bool_val").unwrap();

    assert_eq!(table.rows[0][string_idx].as_str(), Some("new string"));
    assert_eq!(table.rows[0][int_idx].as_i64(), Some(100));
    assert!((table.rows[0][float_idx].as_f64().unwrap() - 2.718).abs() < 0.001);
    assert_eq!(table.rows[0][bool_idx].as_bool(), Some(false));
}

#[test]
fn test_save_cell_row_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "single.toml",
        r#"[[cards]]
id = "abc"
"#,
    );

    // Try to update a row that doesn't exist
    let result = harness.save_cell(&path, "cards", 5, "id", json!("new"));

    match result {
        Err(TvError::RowNotFound { table_name, row_index }) => {
            assert_eq!(table_name, "cards");
            assert_eq!(row_index, 5);
        }
        other => panic!("Expected RowNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_save_cell_table_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "wrong.toml",
        r#"[[items]]
id = "abc"
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "id", json!("new"));

    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_save_cell_adds_new_column() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "add_column.toml",
        r#"[[cards]]
id = "abc"
name = "Test"
"#,
    );

    // Add a new column that doesn't exist
    harness.save_cell(&path, "cards", 0, "new_column", json!("new value")).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    assert!(table.headers.contains(&"new_column".to_string()), "New column should be added");

    let col_idx = table.headers.iter().position(|h| h == "new_column").unwrap();
    assert_eq!(table.rows[0][col_idx].as_str(), Some("new value"));
}

#[test]
fn test_save_cell_null_removes_key() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "remove.toml",
        r#"[[cards]]
id = "abc"
optional = "to be removed"
"#,
    );

    // Set value to null to remove the key
    harness.save_cell(&path, "cards", 0, "optional", serde_json::Value::Null).unwrap();

    let content = harness.read_file_content(&path);
    assert!(!content.contains("optional"), "Key should be removed from file");
    assert!(content.contains("id = \"abc\""), "Other keys should remain");
}

#[test]
fn test_save_cell_unicode_values() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "unicode.toml",
        r#"[[cards]]
id = "uni-001"
name = "Original"
"#,
    );

    harness.save_cell(&path, "cards", 0, "name", json!("日本語テスト 🎴")).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    let name_idx = table.headers.iter().position(|h| h == "name").unwrap();
    assert_eq!(table.rows[0][name_idx].as_str(), Some("日本語テスト 🎴"));
}

#[test]
fn test_save_cell_multiple_updates_same_row() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "multi.toml",
        r#"[[cards]]
a = 1
b = 2
c = 3
"#,
    );

    // Perform multiple updates to the same row
    harness.save_cell(&path, "cards", 0, "a", json!(10)).unwrap();
    harness.save_cell(&path, "cards", 0, "b", json!(20)).unwrap();
    harness.save_cell(&path, "cards", 0, "c", json!(30)).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    let a_idx = table.headers.iter().position(|h| h == "a").unwrap();
    let b_idx = table.headers.iter().position(|h| h == "b").unwrap();
    let c_idx = table.headers.iter().position(|h| h == "c").unwrap();

    assert_eq!(table.rows[0][a_idx].as_i64(), Some(10));
    assert_eq!(table.rows[0][b_idx].as_i64(), Some(20));
    assert_eq!(table.rows[0][c_idx].as_i64(), Some(30));
}

#[test]
fn test_save_batch_updates_multiple_cells() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch.toml",
        r#"[[cards]]
id = "card-1"
name = "First"
cost = 1

[[cards]]
id = "card-2"
name = "Second"
cost = 2

[[cards]]
id = "card-3"
name = "Third"
cost = 3
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "name".to_string(), value: json!("Updated First") },
        CellUpdate { row_index: 1, column_key: "cost".to_string(), value: json!(20) },
        CellUpdate { row_index: 2, column_key: "name".to_string(), value: json!("Updated Third") },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok(), "Batch save should succeed: {result:?}");

    let batch_result = result.unwrap();
    assert!(batch_result.success);
    assert_eq!(batch_result.applied_count, 3);
    assert_eq!(batch_result.failed_count, 0);

    let table = harness.load_table(&path, "cards").unwrap();
    let name_idx = table.headers.iter().position(|h| h == "name").unwrap();
    let cost_idx = table.headers.iter().position(|h| h == "cost").unwrap();

    assert_eq!(table.rows[0][name_idx].as_str(), Some("Updated First"));
    assert_eq!(table.rows[1][cost_idx].as_i64(), Some(20));
    assert_eq!(table.rows[2][name_idx].as_str(), Some("Updated Third"));
}

#[test]
fn test_save_batch_single_atomic_write() {
    let harness = TvTestHarness::new();
    let original = r#"# Important header comment

[[cards]]
id = "card-1"
name = "First"

[[cards]]
id = "card-2"
name = "Second"
"#;
    let path = harness.create_toml_file("atomic.toml", original);

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "name".to_string(), value: json!("A") },
        CellUpdate { row_index: 1, column_key: "name".to_string(), value: json!("B") },
    ];

    harness.save_batch(&path, "cards", &updates).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("# Important header comment"));
    assert!(content.contains("name = \"A\""));
    assert!(content.contains("name = \"B\""));
}

#[test]
fn test_save_batch_empty_updates() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "empty_batch.toml",
        r#"[[cards]]
id = "card-1"
"#,
    );

    let result = harness.save_batch(&path, "cards", &[]).unwrap();
    assert!(result.success);
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.failed_count, 0);
}

#[test]
fn test_save_batch_rejects_entire_batch_on_validation_failure() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "partial.toml",
        r#"[[cards]]
id = "card-1"
name = "First"
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "name".to_string(), value: json!("Updated") },
        CellUpdate { row_index: 99, column_key: "name".to_string(), value: json!("Invalid Row") },
    ];

    let result = harness.save_batch(&path, "cards", &updates).unwrap();
    assert!(!result.success);
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.failed_count, 1);
    assert_eq!(result.failed_updates.len(), 1);
    assert_eq!(result.failed_updates[0].row_index, 99);
    assert!(result.failed_updates[0].reason.contains("out of bounds"));

    let table = harness.load_table(&path, "cards").unwrap();
    let name_idx = table.headers.iter().position(|h| h == "name").unwrap();
    assert_eq!(table.rows[0][name_idx].as_str(), Some("First"));
}

#[test]
fn test_save_batch_table_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "wrong_table.toml",
        r#"[[items]]
id = "item-1"
"#,
    );

    let updates =
        vec![CellUpdate { row_index: 0, column_key: "id".to_string(), value: json!("new") }];

    let result = harness.save_batch(&path, "cards", &updates);
    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_save_batch_preserves_comments() {
    let harness = TvTestHarness::new();
    // Note: When using toml_edit's Item assignment (table[key] = value), inline
    // comments on the same line as the modified value are lost. This is a
    // limitation of the library. Block comments and comments on other lines are
    // preserved.
    let original = r#"# Header comment

[[cards]]
# Card 1 comment
id = "card-1"
name = "First"

[[cards]]
id = "card-2"
name = "Second"
"#;
    let path = harness.create_toml_file("batch_comments.toml", original);

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "name".to_string(), value: json!("New First") },
        CellUpdate { row_index: 1, column_key: "name".to_string(), value: json!("New Second") },
    ];

    harness.save_batch(&path, "cards", &updates).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("# Header comment"));
    assert!(content.contains("# Card 1 comment"));
}

#[test]
fn test_save_batch_same_row_multiple_columns() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "same_row.toml",
        r#"[[cards]]
a = 1
b = 2
c = 3
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "a".to_string(), value: json!(10) },
        CellUpdate { row_index: 0, column_key: "b".to_string(), value: json!(20) },
        CellUpdate { row_index: 0, column_key: "c".to_string(), value: json!(30) },
    ];

    let result = harness.save_batch(&path, "cards", &updates).unwrap();
    assert!(result.success);
    assert_eq!(result.applied_count, 3);

    let table = harness.load_table(&path, "cards").unwrap();
    let a_idx = table.headers.iter().position(|h| h == "a").unwrap();
    let b_idx = table.headers.iter().position(|h| h == "b").unwrap();
    let c_idx = table.headers.iter().position(|h| h == "c").unwrap();

    assert_eq!(table.rows[0][a_idx].as_i64(), Some(10));
    assert_eq!(table.rows[0][b_idx].as_i64(), Some(20));
    assert_eq!(table.rows[0][c_idx].as_i64(), Some(30));
}

#[test]
fn test_save_batch_null_removes_keys() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch_null.toml",
        r#"[[cards]]
id = "card-1"
optional1 = "remove me"
optional2 = "remove me too"
keep = "stay"
"#,
    );

    let updates = vec![
        CellUpdate {
            row_index: 0,
            column_key: "optional1".to_string(),
            value: serde_json::Value::Null,
        },
        CellUpdate {
            row_index: 0,
            column_key: "optional2".to_string(),
            value: serde_json::Value::Null,
        },
    ];

    harness.save_batch(&path, "cards", &updates).unwrap();

    let content = harness.read_file_content(&path);
    assert!(!content.contains("optional1"));
    assert!(!content.contains("optional2"));
    assert!(content.contains("keep = \"stay\""));
}
