use proptest::prelude::*;
use serde_json::json;

use crate::test_utils::test_utils_mod::TvTestHarness;

fn valid_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s.to_string())
}

fn simple_toml_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        any::<i64>().prop_map(|i| json!(i)),
        any::<bool>().prop_map(|b| json!(b)),
        "[a-zA-Z0-9 ]{0,20}".prop_map(|s| json!(s)),
    ]
}

fn toml_row() -> impl Strategy<Value = Vec<(String, serde_json::Value)>> {
    proptest::collection::vec((valid_key(), simple_toml_value()), 1..5).prop_map(|pairs| {
        let mut seen = std::collections::HashSet::new();
        pairs.into_iter().filter(|(k, _)| seen.insert(k.clone())).collect()
    })
}

fn row_to_toml_string(row: &[(String, serde_json::Value)]) -> String {
    let mut result = String::new();
    for (key, value) in row {
        let value_str = match value {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => continue,
        };
        result.push_str(&format!("{} = {}\n", key, value_str));
    }
    result
}

fn generate_toml_content(rows: &[Vec<(String, serde_json::Value)>]) -> String {
    let mut content = String::new();
    for row in rows {
        content.push_str("[[items]]\n");
        content.push_str(&row_to_toml_string(row));
        content.push('\n');
    }
    content
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_roundtrip_preserves_data(rows in proptest::collection::vec(toml_row(), 1..4)) {
        let harness = TvTestHarness::new();
        let content = generate_toml_content(&rows);
        let path = harness.create_toml_file("roundtrip.toml", &content);

        let table1 = harness.load_table(&path, "items").expect("Should load TOML");

        harness.save_table(&path, "items", &table1).expect("Should save TOML");

        let table2 = harness.load_table(&path, "items").expect("Should reload TOML");

        prop_assert_eq!(table1.headers, table2.headers, "Headers should match after round-trip");
        prop_assert_eq!(table1.rows.len(), table2.rows.len(), "Row count should match");

        for (idx, (row1, row2)) in table1.rows.iter().zip(table2.rows.iter()).enumerate() {
            for (col_idx, (val1, val2)) in row1.iter().zip(row2.iter()).enumerate() {
                prop_assert_eq!(
                    val1, val2,
                    "Row {} col {} values should match: {:?} vs {:?}",
                    idx, col_idx, val1, val2
                );
            }
        }
    }

    #[test]
    fn prop_comments_survive_edits(
        initial_value in "[a-zA-Z0-9]{1,10}",
        new_value in "[a-zA-Z0-9]{1,10}"
    ) {
        let header_comment = "# This is a header comment that should survive";
        let block_comment = "# Block comment before the row";
        let content = format!(
            r#"{}

[[items]]
{}
id = "test-id"
name = "{}"
"#,
            header_comment, block_comment, initial_value
        );

        let harness = TvTestHarness::new();
        let path = harness.create_toml_file("comments.toml", &content);

        harness
            .save_cell(&path, "items", 0, "name", json!(new_value.clone()))
            .expect("Should save cell");

        let file_content = harness.read_file_content(&path);

        prop_assert!(
            file_content.contains(header_comment),
            "Header comment should be preserved, got: {}",
            file_content
        );
        prop_assert!(
            file_content.contains(block_comment),
            "Block comment should be preserved, got: {}",
            file_content
        );
        prop_assert!(
            file_content.contains(&new_value),
            "New value should be present, got: {}",
            file_content
        );
    }

    #[test]
    fn prop_whitespace_stable(
        values in proptest::collection::vec("[a-zA-Z0-9]{1,10}".prop_map(String::from), 2..4)
    ) {
        let harness = TvTestHarness::new();

        let mut content = String::new();
        for (i, v) in values.iter().enumerate() {
            content.push_str("[[items]]\n");
            content.push_str(&format!("id = \"id-{}\"\n", i));
            content.push_str(&format!("value = \"{}\"\n\n", v));
        }

        let path = harness.create_toml_file("whitespace.toml", &content);

        harness.save_cell(&path, "items", 0, "value", json!("modified1")).unwrap();
        let content_after_1 = harness.read_file_content(&path);

        harness.save_cell(&path, "items", 0, "value", json!("modified2")).unwrap();
        let content_after_2 = harness.read_file_content(&path);

        harness.save_cell(&path, "items", 0, "value", json!("modified3")).unwrap();
        let content_after_3 = harness.read_file_content(&path);

        let normalize_whitespace = |s: &str| -> String {
            s.replace("modified1", "X")
                .replace("modified2", "X")
                .replace("modified3", "X")
        };

        let norm1 = normalize_whitespace(&content_after_1);
        let norm2 = normalize_whitespace(&content_after_2);
        let norm3 = normalize_whitespace(&content_after_3);

        prop_assert_eq!(
            &norm1, &norm2,
            "Whitespace structure should be stable between saves"
        );
        prop_assert_eq!(
            &norm2, &norm3,
            "Whitespace structure should remain stable after multiple saves"
        );
    }

    #[test]
    fn prop_key_order_preserved(
        key1 in valid_key(),
        key2 in valid_key(),
        key3 in valid_key()
    ) {
        prop_assume!(key1 != key2 && key2 != key3 && key1 != key3);
        prop_assume!(!key1.contains(&key2) && !key2.contains(&key1));
        prop_assume!(!key2.contains(&key3) && !key3.contains(&key2));
        prop_assume!(!key1.contains(&key3) && !key3.contains(&key1));

        let content = format!(
            r#"[[items]]
{} = "value1"
{} = "value2"
{} = "value3"
"#,
            key1, key2, key3
        );

        let harness = TvTestHarness::new();
        let path = harness.create_toml_file("keyorder.toml", &content);

        harness.save_cell(&path, "items", 0, &key2, json!("updated")).unwrap();

        let file_content = harness.read_file_content(&path);

        let pattern1 = format!("{} = ", key1);
        let pattern2 = format!("{} = ", key2);
        let pattern3 = format!("{} = ", key3);

        let pos1 = file_content.find(&pattern1).expect("key1 pattern should exist");
        let pos2 = file_content.find(&pattern2).expect("key2 pattern should exist");
        let pos3 = file_content.find(&pattern3).expect("key3 pattern should exist");

        prop_assert!(
            pos1 < pos2 && pos2 < pos3,
            "Key order should be preserved: {} < {} < {}, positions: {} < {} < {}",
            key1, key2, key3, pos1, pos2, pos3
        );
    }

    #[test]
    fn prop_multiple_row_edits_preserve_structure(
        num_rows in 2..5usize,
        edit_row in 0..4usize
    ) {
        let edit_row = edit_row % num_rows;

        let harness = TvTestHarness::new();
        let mut content = String::new();
        for i in 0..num_rows {
            content.push_str(&format!(
                r#"[[items]]
id = "row-{}"
name = "Name {}"
count = {}

"#,
                i, i, i * 10
            ));
        }

        let path = harness.create_toml_file("multirow.toml", &content);

        let table_before = harness.load_table(&path, "items").expect("Should load");
        prop_assert_eq!(table_before.rows.len(), num_rows);

        harness.save_cell(&path, "items", edit_row, "name", json!("Modified Name")).unwrap();

        let table_after = harness.load_table(&path, "items").expect("Should reload");
        prop_assert_eq!(table_after.rows.len(), num_rows);

        for row_idx in 0..num_rows {
            let id_idx = table_after.headers.iter().position(|h| h == "id").unwrap();
            let expected_id = format!("row-{}", row_idx);
            prop_assert_eq!(
                table_after.rows[row_idx][id_idx].as_str(),
                Some(expected_id.as_str()),
                "Row {} ID should be preserved",
                row_idx
            );
        }

        let name_idx = table_after.headers.iter().position(|h| h == "name").unwrap();
        prop_assert_eq!(
            table_after.rows[edit_row][name_idx].as_str(),
            Some("Modified Name"),
            "Edited row should have new value"
        );
    }

    #[test]
    fn prop_data_types_preserved_through_roundtrip(
        int_val in any::<i32>().prop_map(|i| i as i64),
        bool_val in any::<bool>(),
        str_val in "[a-zA-Z0-9 ]{1,20}"
    ) {
        let content = format!(
            r#"[[items]]
int_field = {}
bool_field = {}
str_field = "{}"
"#,
            int_val, bool_val, str_val
        );

        let harness = TvTestHarness::new();
        let path = harness.create_toml_file("types.toml", &content);

        let table1 = harness.load_table(&path, "items").expect("Should load");
        harness.save_table(&path, "items", &table1).expect("Should save");
        let table2 = harness.load_table(&path, "items").expect("Should reload");

        let int_idx = table2.headers.iter().position(|h| h == "int_field").unwrap();
        let bool_idx = table2.headers.iter().position(|h| h == "bool_field").unwrap();
        let str_idx = table2.headers.iter().position(|h| h == "str_field").unwrap();

        prop_assert_eq!(
            table2.rows[0][int_idx].as_i64(),
            Some(int_val),
            "Integer should be preserved"
        );
        prop_assert_eq!(
            table2.rows[0][bool_idx].as_bool(),
            Some(bool_val),
            "Boolean should be preserved"
        );
        prop_assert_eq!(
            table2.rows[0][str_idx].as_str(),
            Some(str_val.as_str()),
            "String should be preserved"
        );
    }
}

#[test]
fn test_inline_table_preservation() {
    let content = r#"[[items]]
id = "test"
config = { enabled = true, count = 5 }
"#;

    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("inline_table.toml", content);

    harness.save_cell(&path, "items", 0, "id", json!("modified")).unwrap();

    let file_content = harness.read_file_content(&path);
    assert!(
        file_content.contains("config = { enabled = true, count = 5 }")
            || file_content.contains("config = {enabled = true, count = 5}")
            || (file_content.contains("enabled = true") && file_content.contains("count = 5")),
        "Inline table should be preserved, got: {}",
        file_content
    );
}

#[test]
fn test_array_value_preservation() {
    let content = r#"[[items]]
id = "test"
tags = ["alpha", "beta", "gamma"]
"#;

    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("array_value.toml", content);

    harness.save_cell(&path, "items", 0, "id", json!("modified")).unwrap();

    let table = harness.load_table(&path, "items").expect("Should load");
    let tags_idx = table.headers.iter().position(|h| h == "tags").unwrap();
    let tags = &table.rows[0][tags_idx];

    assert!(tags.is_array(), "Tags should still be an array");
    let arr = tags.as_array().unwrap();
    assert_eq!(arr.len(), 3, "Array should have 3 elements");
    assert_eq!(arr[0].as_str(), Some("alpha"));
    assert_eq!(arr[1].as_str(), Some("beta"));
    assert_eq!(arr[2].as_str(), Some("gamma"));
}

#[test]
fn test_empty_string_preservation() {
    let content = r#"[[items]]
id = "test"
empty = ""
"#;

    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("empty_string.toml", content);

    let table1 = harness.load_table(&path, "items").expect("Should load");
    harness.save_table(&path, "items", &table1).expect("Should save");
    let table2 = harness.load_table(&path, "items").expect("Should reload");

    let empty_idx = table2.headers.iter().position(|h| h == "empty").unwrap();
    assert_eq!(table2.rows[0][empty_idx].as_str(), Some(""));
}

#[test]
fn test_unicode_preservation() {
    let content = r#"[[items]]
id = "test"
japanese = "日本語"
emoji = "Hello World"
cyrillic = "Привет"
"#;

    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("unicode.toml", content);

    harness.save_cell(&path, "items", 0, "id", json!("modified")).unwrap();

    let table = harness.load_table(&path, "items").expect("Should load");
    let japanese_idx = table.headers.iter().position(|h| h == "japanese").unwrap();
    let cyrillic_idx = table.headers.iter().position(|h| h == "cyrillic").unwrap();

    assert_eq!(table.rows[0][japanese_idx].as_str(), Some("日本語"));
    assert_eq!(table.rows[0][cyrillic_idx].as_str(), Some("Привет"));
}

#[test]
fn test_multiline_string_comment_pattern() {
    let content = r#"# File header
# Second line of header

[[items]]
# Item comment
id = "test"
description = "A simple description"
"#;

    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("multiline_comment.toml", content);

    harness.save_cell(&path, "items", 0, "description", json!("Updated description")).unwrap();

    let file_content = harness.read_file_content(&path);
    assert!(file_content.contains("# File header"));
    assert!(file_content.contains("# Second line of header"));
    assert!(file_content.contains("# Item comment"));
    assert!(file_content.contains("Updated description"));
}
