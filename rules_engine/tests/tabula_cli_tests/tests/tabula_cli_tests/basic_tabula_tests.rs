use tabula_cli::core::excel_reader::{self, ColumnType};
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;

#[test]
fn test_extract_tables_finds_named_table() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_table.xlsx");

    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path)
        .expect("Failed to create test spreadsheet");

    let tables = excel_reader::extract_tables(&xlsx_path).expect("Failed to extract tables");

    assert_eq!(tables.len(), 1, "Expected 1 table");
    assert_eq!(tables[0].name, "TestTable");
    assert!(!tables[0].columns.is_empty(), "Expected columns in table");
    assert!(!tables[0].rows.is_empty(), "Expected rows in table");
}

#[test]
fn test_column_classification_data_and_empty() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_columns.xlsx");

    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path)
        .expect("Failed to create test spreadsheet");

    let tables = excel_reader::extract_tables(&xlsx_path).expect("Failed to extract tables");
    let table = &tables[0];

    let name_col = table.columns.iter().find(|c| c.name == "Name").expect("Name column");
    let count_col = table.columns.iter().find(|c| c.name == "Count").expect("Count column");
    let active_col = table.columns.iter().find(|c| c.name == "Active").expect("Active column");
    let empty_col = table.columns.iter().find(|c| c.name == "Empty").expect("Empty column");

    assert_eq!(name_col.column_type, ColumnType::Data);
    assert_eq!(count_col.column_type, ColumnType::Data);
    assert_eq!(active_col.column_type, ColumnType::Data);
    assert_eq!(empty_col.column_type, ColumnType::Empty);
}

#[test]
fn test_cell_value_extraction() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_values.xlsx");

    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path)
        .expect("Failed to create test spreadsheet");

    let tables = excel_reader::extract_tables(&xlsx_path).expect("Failed to extract tables");
    let table = &tables[0];

    assert_eq!(table.rows.len(), 2, "Expected 2 data rows");

    let row1 = &table.rows[0];
    assert!(row1.contains_key("Name"), "Expected Name in row");
    assert!(row1.contains_key("Count"), "Expected Count in row");
    assert!(row1.contains_key("Active"), "Expected Active in row");
}

#[test]
fn test_error_on_no_tables() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("no_table.xlsx");

    tabula_cli_test_utils::create_simple_spreadsheet(&xlsx_path)
        .expect("Failed to create spreadsheet");

    let result = excel_reader::extract_tables(&xlsx_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No named Excel Tables"));
}
