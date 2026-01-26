use serde_json::json;
use tv_lib::toml::document_loader::TomlTableData;
use tv_lib::toml::document_writer::CellUpdate;

use crate::test_utils::test_utils_mod::TvTestHarness;

#[test]
fn test_delete_row_preserves_boolean_types() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_delete.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
art-owned = true

[[cards]]
name = "Card 2"
is-fast = true
art-owned = false
"#,
    );

    let result = harness.delete_row(&path, "cards", 0);
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = true"),
        "Expected 'is-fast = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = false"),
        "Expected 'art-owned = false' but got:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("is-fast = 1"),
        "Boolean was converted to integer:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("is-fast = 0"),
        "Boolean was converted to integer:\n{saved_content}",
    );
}

#[test]
fn test_add_row_preserves_boolean_types_in_existing_rows() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_add.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
art-owned = true
"#,
    );

    let result = harness.add_row(&path, "cards", None, None);
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = false"),
        "Expected 'is-fast = false' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = true"),
        "Expected 'art-owned = true' but got:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("is-fast = 0"),
        "Boolean was converted to integer:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("art-owned = 1"),
        "Boolean was converted to integer:\n{saved_content}",
    );
}

#[test]
fn test_save_toml_document_preserves_boolean_types() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_save.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
art-owned = true

[[cards]]
name = "Card 2"
is-fast = true
art-owned = false
"#,
    );

    let data = TomlTableData {
        headers: vec!["name".to_string(), "is-fast".to_string(), "art-owned".to_string()],
        rows: vec![vec![json!("Card 1"), json!(false), json!(true)], vec![
            json!("Card 2"),
            json!(true),
            json!(false),
        ]],
    };

    let result = harness.save_table(&path, "cards", &data);
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = false"),
        "Expected 'is-fast = false' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("is-fast = true"),
        "Expected 'is-fast = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = true"),
        "Expected 'art-owned = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = false"),
        "Expected 'art-owned = false' but got:\n{saved_content}",
    );
    assert!(!saved_content.contains("= 0"), "Boolean was converted to integer 0:\n{saved_content}",);
    assert!(!saved_content.contains("= 1"), "Boolean was converted to integer 1:\n{saved_content}",);
}

#[test]
fn test_save_toml_document_converts_integers_back_to_booleans() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_int_convert.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
art-owned = true

[[cards]]
name = "Card 2"
is-fast = true
art-owned = false
"#,
    );

    let data = TomlTableData {
        headers: vec!["name".to_string(), "is-fast".to_string(), "art-owned".to_string()],
        rows: vec![vec![json!("Card 1"), json!(0), json!(1)], vec![
            json!("Card 2"),
            json!(1),
            json!(0),
        ]],
    };

    let result = harness.save_table(&path, "cards", &data);
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = false"),
        "Expected 'is-fast = false' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("is-fast = true"),
        "Expected 'is-fast = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = true"),
        "Expected 'art-owned = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = false"),
        "Expected 'art-owned = false' but got:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("= 0"),
        "Integer 0 should have been converted to boolean false:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("= 1"),
        "Integer 1 should have been converted to boolean true:\n{saved_content}",
    );
}

#[test]
fn test_save_cell_converts_integer_back_to_boolean() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_cell.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
"#,
    );

    let result = harness.save_cell(&path, "cards", 0, "is-fast", json!(1));
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = true"),
        "Expected 'is-fast = true' but got:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("is-fast = 1"),
        "Integer should have been converted to boolean:\n{saved_content}",
    );
}

#[test]
fn test_save_batch_converts_integers_back_to_booleans() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "bool_batch.toml",
        r#"[[cards]]
name = "Card 1"
is-fast = false
art-owned = true
"#,
    );

    let updates = vec![
        CellUpdate { row_index: 0, column_key: "is-fast".to_string(), value: json!(1) },
        CellUpdate { row_index: 0, column_key: "art-owned".to_string(), value: json!(0) },
    ];

    let result = harness.save_batch(&path, "cards", &updates);
    assert!(result.is_ok());

    let saved_content = harness.read_file_content(&path);

    assert!(
        saved_content.contains("is-fast = true"),
        "Expected 'is-fast = true' but got:\n{saved_content}",
    );
    assert!(
        saved_content.contains("art-owned = false"),
        "Expected 'art-owned = false' but got:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("= 0"),
        "Integer should have been converted to boolean:\n{saved_content}",
    );
    assert!(
        !saved_content.contains("= 1"),
        "Integer should have been converted to boolean:\n{saved_content}",
    );
}
