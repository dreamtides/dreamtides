use serde_json::json;
use tv_lib::toml::document_writer::CellUpdate;

use crate::test_utils::harness::TvTestHarness;

#[test]
fn test_load_with_inline_arrays() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "arrays.toml",
        r#"
[[cards]]
name = "Card A"
tags = ["a", "b", "c"]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");
    assert!(table.headers.contains(&"tags[0]".to_string()));
    assert!(table.headers.contains(&"tags[1]".to_string()));
    assert!(table.headers.contains(&"tags[2]".to_string()));
    assert!(!table.headers.contains(&"tags".to_string()));

    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "tags[1]").unwrap();
    let idx2 = table.headers.iter().position(|h| h == "tags[2]").unwrap();

    assert_eq!(table.rows[0][idx0].as_str(), Some("a"));
    assert_eq!(table.rows[0][idx1].as_str(), Some("b"));
    assert_eq!(table.rows[0][idx2].as_str(), Some("c"));
}

#[test]
fn test_load_with_varying_lengths() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "varying.toml",
        r#"
[[cards]]
name = "Short"
tags = ["x", "y"]

[[cards]]
name = "Long"
tags = ["a", "b", "c"]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");

    // Max length is 3, so we should have tags[0], tags[1], tags[2]
    assert!(table.headers.contains(&"tags[2]".to_string()));

    let idx2 = table.headers.iter().position(|h| h == "tags[2]").unwrap();
    assert!(table.rows[0][idx2].is_null(), "Short row should have null for tags[2]");
    assert_eq!(table.rows[1][idx2].as_str(), Some("c"));
}

#[test]
fn test_load_with_empty_array() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "empty_arr.toml",
        r#"
[[cards]]
name = "Has tags"
tags = ["a", "b"]

[[cards]]
name = "No tags"
tags = []
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");

    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "tags[1]").unwrap();

    assert_eq!(table.rows[0][idx0].as_str(), Some("a"));
    assert_eq!(table.rows[0][idx1].as_str(), Some("b"));
    assert!(table.rows[1][idx0].is_null());
    assert!(table.rows[1][idx1].is_null());
}

#[test]
fn test_save_cell_to_array_element() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "cell_array.toml",
        r#"
[[cards]]
name = "Card"
tags = ["old_a", "old_b"]
"#,
    );

    harness.save_cell(&path, "cards", 0, "tags[1]", json!("new_b")).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("new_b"), "Updated value should appear in file");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "tags[1]").unwrap();
    assert_eq!(table.rows[0][idx0].as_str(), Some("old_a"));
    assert_eq!(table.rows[0][idx1].as_str(), Some("new_b"));
}

#[test]
fn test_save_cell_extends_array() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "extend.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b"]
"#,
    );

    harness.save_cell(&path, "cards", 0, "tags[2]", json!("c")).unwrap();

    let table = harness.load_table(&path, "cards").unwrap();
    let idx2 = table.headers.iter().position(|h| h == "tags[2]").unwrap();
    assert_eq!(table.rows[0][idx2].as_str(), Some("c"));
}

#[test]
fn test_save_cell_clears_element() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "clear.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b", "c"]
"#,
    );

    harness.save_cell(&path, "cards", 0, "tags[0]", serde_json::Value::Null).unwrap();

    let content = harness.read_file_content(&path);
    // After removing index 0, the array should compact to ["b", "c"]
    assert!(!content.contains("\"a\""), "Removed element should not appear");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    assert_eq!(table.rows[0][idx0].as_str(), Some("b"), "Array should compact after removal");
}

#[test]
fn test_full_table_save_round_trip() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "round_trip.toml",
        r#"
[[cards]]
name = "Card A"
tags = ["x", "y"]

[[cards]]
name = "Card B"
tags = ["a", "b", "c"]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");
    harness.save_table(&path, "cards", &table).expect("Should save");
    let reloaded = harness.load_table(&path, "cards").expect("Should reload");

    assert_eq!(table.headers, reloaded.headers);
    assert_eq!(table.rows.len(), reloaded.rows.len());
    for (row_idx, (orig, reload)) in table.rows.iter().zip(reloaded.rows.iter()).enumerate() {
        for (col_idx, (a, b)) in orig.iter().zip(reload.iter()).enumerate() {
            assert_eq!(a, b, "Mismatch at row {row_idx}, col {col_idx}");
        }
    }
}

#[test]
fn test_mixed_columns() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "mixed.toml",
        r#"
[[cards]]
name = "Card A"
cost = 3
tags = ["fire", "ice"]
rarity = "common"
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");

    // Regular columns should exist
    assert!(table.headers.contains(&"name".to_string()));
    assert!(table.headers.contains(&"cost".to_string()));
    assert!(table.headers.contains(&"rarity".to_string()));

    // Array columns should be expanded
    assert!(table.headers.contains(&"tags[0]".to_string()));
    assert!(table.headers.contains(&"tags[1]".to_string()));

    // "tags" raw key should not exist
    assert!(!table.headers.contains(&"tags".to_string()));

    let name_idx = table.headers.iter().position(|h| h == "name").unwrap();
    let cost_idx = table.headers.iter().position(|h| h == "cost").unwrap();
    let tags0_idx = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let tags1_idx = table.headers.iter().position(|h| h == "tags[1]").unwrap();
    let rarity_idx = table.headers.iter().position(|h| h == "rarity").unwrap();

    assert_eq!(table.rows[0][name_idx].as_str(), Some("Card A"));
    assert_eq!(table.rows[0][cost_idx].as_i64(), Some(3));
    assert_eq!(table.rows[0][tags0_idx].as_str(), Some("fire"));
    assert_eq!(table.rows[0][tags1_idx].as_str(), Some("ice"));
    assert_eq!(table.rows[0][rarity_idx].as_str(), Some("common"));
}

#[test]
fn test_batch_save_with_array_columns() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "batch_arr.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b", "c"]
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "tags[0]".to_string(), value: json!("x") },
        CellUpdate { row_index: 0, column_key: "tags[2]".to_string(), value: json!("z") },
    ];

    let result = harness.save_batch(&path, "cards", &updates).unwrap();
    assert!(result.success);
    assert_eq!(result.applied_count, 2);

    let table = harness.load_table(&path, "cards").unwrap();
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "tags[1]").unwrap();
    let idx2 = table.headers.iter().position(|h| h == "tags[2]").unwrap();
    assert_eq!(table.rows[0][idx0].as_str(), Some("x"));
    assert_eq!(table.rows[0][idx1].as_str(), Some("b"));
    assert_eq!(table.rows[0][idx2].as_str(), Some("z"));
}

#[test]
fn test_full_table_save_modifies_array_element() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "table_modify.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b"]
"#,
    );

    let mut table = harness.load_table(&path, "cards").expect("Should load");
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    table.rows[0][idx0] = json!("modified");

    harness.save_table(&path, "cards", &table).expect("Should save");

    let content = harness.read_file_content(&path);
    assert!(content.contains("modified"));
    assert!(content.contains("\"b\""));
}

#[test]
fn test_full_table_save_clears_array_via_all_nulls() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "clear_all.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b"]
"#,
    );

    let mut table = harness.load_table(&path, "cards").expect("Should load");
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "tags[1]").unwrap();
    table.rows[0][idx0] = serde_json::Value::Null;
    table.rows[0][idx1] = serde_json::Value::Null;

    harness.save_table(&path, "cards", &table).expect("Should save");

    let content = harness.read_file_content(&path);
    assert!(!content.contains("tags"), "Array key should be removed when all elements are null");
    assert!(content.contains("name = \"Card\""));
}

#[test]
fn test_integer_array_expansion() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "int_arr.toml",
        r#"
[[cards]]
name = "Card"
values = [10, 20, 30]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");
    let idx0 = table.headers.iter().position(|h| h == "values[0]").unwrap();
    let idx1 = table.headers.iter().position(|h| h == "values[1]").unwrap();
    let idx2 = table.headers.iter().position(|h| h == "values[2]").unwrap();

    assert_eq!(table.rows[0][idx0].as_i64(), Some(10));
    assert_eq!(table.rows[0][idx1].as_i64(), Some(20));
    assert_eq!(table.rows[0][idx2].as_i64(), Some(30));
}

#[test]
fn test_array_of_tables_not_expanded() {
    let harness = TvTestHarness::new();
    // An array containing tables should NOT be expanded — it should be kept
    // as a single JSON cell value.
    let path = harness.create_toml_file(
        "table_arr.toml",
        r#"
[[cards]]
name = "Card"
nested = [{key = "a"}, {key = "b"}]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");
    // "nested" should remain as a single column, not expanded
    assert!(table.headers.contains(&"nested".to_string()));
    assert!(!table.headers.contains(&"nested[0]".to_string()));
}

#[test]
fn test_save_cell_creates_array_from_scratch() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "new_array.toml",
        r#"
[[cards]]
name = "Card"
"#,
    );

    // Writing to tags[0] when no "tags" key exists should create the array
    harness.save_cell(&path, "cards", 0, "tags[0]", json!("hello")).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("tags"), "tags key should be created");

    let table = harness.load_table(&path, "cards").unwrap();
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    assert_eq!(table.rows[0][idx0].as_str(), Some("hello"));
}

#[test]
fn test_save_cell_clear_all_removes_array_key() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "clear_single.toml",
        r#"
[[cards]]
name = "Card"
tags = ["only"]
"#,
    );

    harness.save_cell(&path, "cards", 0, "tags[0]", serde_json::Value::Null).unwrap();

    let content = harness.read_file_content(&path);
    assert!(!content.contains("tags"), "Empty array key should be removed");
}

#[test]
fn test_round_trip_preserves_formatting() {
    let harness = TvTestHarness::new();
    let original = r#"# Card data
[[cards]]
name = "Card A"
tags = ["fire", "ice"]
cost = 3
"#;
    let path = harness.create_toml_file("format.toml", original);

    let table = harness.load_table(&path, "cards").unwrap();
    harness.save_table(&path, "cards", &table).unwrap();

    let content = harness.read_file_content(&path);
    assert!(content.contains("# Card data"), "Comments should be preserved");
    assert!(content.contains("name = \"Card A\""));
    assert!(content.contains("cost = 3"));
}

#[test]
fn test_multiple_array_columns() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "multi_arr.toml",
        r#"
[[cards]]
name = "Card"
tags = ["a", "b"]
colors = ["red", "blue", "green"]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");

    assert!(table.headers.contains(&"tags[0]".to_string()));
    assert!(table.headers.contains(&"tags[1]".to_string()));
    assert!(table.headers.contains(&"colors[0]".to_string()));
    assert!(table.headers.contains(&"colors[1]".to_string()));
    assert!(table.headers.contains(&"colors[2]".to_string()));

    let c0 = table.headers.iter().position(|h| h == "colors[0]").unwrap();
    let c1 = table.headers.iter().position(|h| h == "colors[1]").unwrap();
    let c2 = table.headers.iter().position(|h| h == "colors[2]").unwrap();
    assert_eq!(table.rows[0][c0].as_str(), Some("red"));
    assert_eq!(table.rows[0][c1].as_str(), Some("blue"));
    assert_eq!(table.rows[0][c2].as_str(), Some("green"));
}

#[test]
fn test_no_array_key_yields_no_columns_on_load() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "no_arr.toml",
        r#"
[[cards]]
name = "Card A"
cost = 3

[[cards]]
name = "Card B"
tags = ["x"]
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load");

    // Row 0 does not have "tags", so tags[0] should be null
    let idx0 = table.headers.iter().position(|h| h == "tags[0]").unwrap();
    assert!(table.rows[0][idx0].is_null());
    assert_eq!(table.rows[1][idx0].as_str(), Some("x"));
}
