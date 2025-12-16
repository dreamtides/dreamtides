use std::fs;

use tabula_cli::commands::build_toml;
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use toml::Value;

#[test]
fn build_toml_writes_table_rows_and_skips_formulas() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir.clone())).expect("build-toml");

    let toml_path = output_dir.join("test-table.toml");
    let content = fs::read_to_string(&toml_path).expect("read toml");
    let value: Value = toml::from_str(&content).expect("parse toml");

    let rows = value.get("test-table").and_then(Value::as_array).cloned().unwrap_or_default();

    assert_eq!(rows.len(), 2);

    let first = rows[0].as_table().expect("first row");
    assert_eq!(first.get("name").and_then(Value::as_str), Some("Alice"));
    assert_eq!(first.get("count").and_then(Value::as_integer), Some(10));
    assert_eq!(first.get("active").and_then(Value::as_bool), Some(true));
    assert!(first.get("computed").is_none());
    assert!(first.get("empty").is_none());
}

#[test]
fn build_toml_creates_backup_copy() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir)).expect("build-toml");

    let backup_dir = git_dir.join("excel-backups");
    let backups: Vec<_> = fs::read_dir(&backup_dir).expect("backup dir").collect();
    assert!(!backups.is_empty());
}

#[test]
fn build_toml_preserves_column_order() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir.clone())).expect("build-toml");

    let toml_path = output_dir.join("test-table.toml");
    let content = fs::read_to_string(&toml_path).expect("read toml");
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if line.starts_with("[[test-table]]") {
            break;
        }
    }
    let mut keys = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        if let Some((key, _)) = line.split_once('=') {
            keys.push(key.trim().to_string());
        }
    }

    assert_eq!(keys, vec!["name".to_string(), "count".to_string(), "active".to_string()]);
}

#[test]
fn build_toml_strips_special_characters_in_column_names() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("special.xlsx");
    tabula_cli_test_utils::create_special_column_spreadsheet(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir.clone())).expect("build-toml");

    let toml_path = output_dir.join("special.toml");
    let content = fs::read_to_string(&toml_path).expect("read toml");
    let value: Value = toml::from_str(&content).expect("parse toml");

    let rows = value.get("special").and_then(Value::as_array).cloned().unwrap_or_default();

    let row = rows[0].as_table().expect("row");
    assert!(row.contains_key("is-fast"));
    assert!(!row.contains_key("is-fast?"));
}

#[test]
fn build_toml_encodes_single_column_table_as_array() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("predicate_types.xlsx");
    tabula_cli_test_utils::create_predicate_types_spreadsheet(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir.clone())).expect("build-toml");

    let toml_path = output_dir.join("predicate-types.toml");
    let content = fs::read_to_string(&toml_path).expect("read toml");
    let value: Value = toml::from_str(&content).expect("parse toml");

    let array = value.get("predicate_types").and_then(Value::as_array).cloned().unwrap_or_default();
    let strings: Vec<String> =
        array.iter().filter_map(Value::as_str).map(ToString::to_string).collect();
    assert_eq!(strings, vec![
        "ThisCard".to_string(),
        "ForEachTarget".to_string(),
        "ControllerDeck".to_string()
    ]);
}

#[test]
fn build_toml_prunes_old_backups() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    let backup_dir = git_dir.join("excel-backups");
    fs::create_dir_all(&backup_dir).expect("git dir");

    for i in 0..60 {
        let name = format!("202001010000{0:02}-test.xlsm", i);
        fs::write(backup_dir.join(name), b"old").expect("seed backup");
    }

    let xlsx_path = temp_dir.path().join("table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let output_dir = temp_dir.path().join("out");
    build_toml::build_toml(Some(xlsx_path), Some(output_dir)).expect("build-toml");

    let backups: Vec<_> = fs::read_dir(&backup_dir).expect("backup dir").collect();
    assert_eq!(backups.len(), 50);
    assert!(!backup_dir.join("20200101000000-test.xlsm").exists());
    assert!(backup_dir.join("20200101000059-test.xlsm").exists());
}
