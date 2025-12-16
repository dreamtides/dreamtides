use std::collections::BTreeMap;

use old_tabula_cli::spreadsheet::{SheetRow, SheetTable, SheetValue, Spreadsheet};
use old_tabula_cli_tests::old_tabula_cli_test_utils::FakeSpreadsheet;
use serde_json::Value;

#[tokio::test]
async fn read_and_write_single_cell() {
    let fake = FakeSpreadsheet::default();
    fake.write_cell("Sheet1", "B", 2, "hello").await.unwrap();
    let v = fake.read_cell("Sheet1", "B", 2).await.unwrap();
    assert_eq!(v.as_deref(), Some("hello"));
}

#[tokio::test]
async fn read_empty_returns_none() {
    let fake = FakeSpreadsheet::default();
    let v = fake.read_cell("SheetX", "A", 1).await.unwrap();
    assert!(v.is_none());
}

#[tokio::test]
async fn write_and_read_table() {
    let table = {
        let mut row1 = BTreeMap::new();
        row1.insert("Col1".to_string(), SheetValue { data: Value::String("x".to_string()) });
        row1.insert("Col2".to_string(), SheetValue { data: Value::String("1".to_string()) });
        let mut row2 = BTreeMap::new();
        row2.insert("Col1".to_string(), SheetValue { data: Value::String("y".to_string()) });
        row2.insert("Col2".to_string(), SheetValue { data: Value::String("2".to_string()) });
        SheetTable {
            name: "SheetT".to_string(),
            rows: vec![SheetRow { values: row1 }, SheetRow { values: row2 }],
        }
    };
    let fake = FakeSpreadsheet::default();
    fake.write_table(&table).await.unwrap();
    let round = fake.read_table("SheetT").await.unwrap();
    assert_eq!(round.rows.len(), 2);
    assert_eq!(round.rows[1].values.get("Col1").unwrap().data, Value::String("y".to_string()));
    assert_eq!(round.rows[1].values.get("Col2").unwrap().data, Value::String("2".to_string()));
}

#[test]
fn construct_default_fake() {
    let _spreadsheet = FakeSpreadsheet::default();
}
