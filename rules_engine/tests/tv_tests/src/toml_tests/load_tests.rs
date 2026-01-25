use tv_lib::error::error_types::TvError;

use crate::test_utils::test_utils_mod::TvTestHarness;

#[test]
fn test_load_simple_table() {
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

    let table = harness.load_table(&path, "cards").expect("Should load valid TOML file");

    assert_eq!(table.rows.len(), 2, "Should have 2 rows");
    assert_eq!(table.headers.len(), 3, "Should have 3 columns: id, name, cost");
    assert!(table.headers.contains(&"id".to_string()), "Headers should contain 'id'");
    assert!(table.headers.contains(&"name".to_string()), "Headers should contain 'name'");
    assert!(table.headers.contains(&"cost".to_string()), "Headers should contain 'cost'");

    let id_idx = table.headers.iter().position(|h| h == "id").expect("id column should exist");
    assert_eq!(table.rows[0][id_idx].as_str(), Some("abc-123"), "First row id should be 'abc-123'");
    assert_eq!(
        table.rows[1][id_idx].as_str(),
        Some("def-456"),
        "Second row id should be 'def-456'"
    );
}

#[test]
fn test_load_sparse_data() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "sparse.toml",
        r#"
[[cards]]
id = "abc"
name = "Card One"

[[cards]]
id = "def"
cost = 5
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load sparse TOML file");

    assert_eq!(
        table.headers.len(),
        3,
        "Headers should include all keys from all rows: id, name, cost"
    );
    assert!(table.headers.contains(&"id".to_string()), "Headers should contain 'id'");
    assert!(
        table.headers.contains(&"name".to_string()),
        "Headers should contain 'name' from first row"
    );
    assert!(
        table.headers.contains(&"cost".to_string()),
        "Headers should contain 'cost' from second row"
    );
}

#[test]
fn test_load_file_not_found() {
    let harness = TvTestHarness::new();
    let nonexistent = harness.temp_dir().join("nonexistent.toml");

    let result = harness.load_table(&nonexistent, "cards");

    match result {
        Err(TvError::FileNotFound { path }) => {
            assert!(
                path.contains("nonexistent.toml"),
                "Error path should include filename, got: {path}"
            );
        }
        other => panic!("Expected FileNotFound error, got: {other:?}"),
    }
}

#[test]
fn test_load_parse_error() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "invalid.toml",
        r#"
[[cards]]
id = "abc"
name = missing quotes
"#,
    );

    let result = harness.load_table(&path, "cards");

    match result {
        Err(TvError::TomlParseError { line, message, .. }) => {
            assert!(line.is_some(), "Parse error should include line number");
            assert!(!message.is_empty(), "Parse error should include message");
        }
        other => panic!("Expected TomlParseError, got: {other:?}"),
    }
}

#[test]
fn test_load_unicode() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "unicode.toml",
        r#"
[[cards]]
id = "uni-001"
name = "日本語カード"
description = "Описание карты 🎴"
"#,
    );

    let table = harness.load_table(&path, "cards").expect("Should load unicode TOML file");

    let name_idx =
        table.headers.iter().position(|h| h == "name").expect("name column should exist");
    let desc_idx = table
        .headers
        .iter()
        .position(|h| h == "description")
        .expect("description column should exist");

    assert_eq!(
        table.rows[0][name_idx].as_str(),
        Some("日本語カード"),
        "Japanese text should be preserved"
    );
    assert_eq!(
        table.rows[0][desc_idx].as_str(),
        Some("Описание карты 🎴"),
        "Russian text with emoji should be preserved"
    );
}

#[test]
fn test_load_table_not_found() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file(
        "wrong_table.toml",
        r#"
[[items]]
id = "abc"
"#,
    );

    let result = harness.load_table(&path, "cards");

    match result {
        Err(TvError::TableNotFound { table_name }) => {
            assert_eq!(table_name, "cards", "Error should include the requested table name");
        }
        other => panic!("Expected TableNotFound error, got: {other:?}"),
    }
}
