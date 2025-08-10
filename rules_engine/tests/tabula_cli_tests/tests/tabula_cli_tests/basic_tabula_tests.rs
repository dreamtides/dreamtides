use serde_json::Value;
use tabula_cli::spreadsheet::{SheetColumn, SheetTable, SheetValue, Spreadsheet};
use tabula_cli_tests::tabula_cli_test_utils::FakeSpreadsheet;

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
    let table = SheetTable {
        name: "SheetT".to_string(),
        columns: vec![
            SheetColumn {
                name: "Col1".to_string(),
                values: vec![SheetValue { data: Value::String("x".to_string()) }, SheetValue {
                    data: Value::String("y".to_string()),
                }],
            },
            SheetColumn {
                name: "Col2".to_string(),
                values: vec![SheetValue { data: Value::String("1".to_string()) }, SheetValue {
                    data: Value::String("2".to_string()),
                }],
            },
        ],
    };
    let fake = FakeSpreadsheet::default();
    fake.write_table(&table).await.unwrap();
    let round = fake.read_table("SheetT").await.unwrap();
    assert_eq!(round.columns.len(), 2);
    assert_eq!(round.columns[0].name, "Col1");
    assert_eq!(round.columns[1].name, "Col2");
    assert_eq!(round.columns[0].values[1].data, Value::String("y".to_string()));
    assert_eq!(round.columns[1].values[1].data, Value::String("2".to_string()));
}

#[test]
fn construct_default_fake() {
    let _spreadsheet = FakeSpreadsheet::default();
}
