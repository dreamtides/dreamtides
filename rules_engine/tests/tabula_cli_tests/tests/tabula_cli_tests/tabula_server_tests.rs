use tabula_cli::server::listener_runner::{Listener, ListenerContext};
use tabula_cli::server::listeners::conditional_formatting::ConditionalFormattingListener;
use tabula_cli::server::model::{Change, ChangedRange};
use tabula_cli::server::server_workbook_snapshot::read_snapshot;
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;

#[test]
fn test_conditional_formatting_finds_pineapple() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_cards.xlsx");

    tabula_cli_test_utils::create_cards_sheet_with_pineapple(&xlsx_path)
        .expect("Failed to create test spreadsheet");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-1".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = ConditionalFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let bold_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetBold { sheet, cell, bold } if *bold => Some((sheet.clone(), cell.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(bold_changes.len(), 2, "Expected 2 cells with pineapple");
    assert!(
        bold_changes.iter().any(|(s, c)| s == "Cards" && c == "A3"),
        "Should find 'Pineapple' in A3"
    );
    assert!(
        bold_changes.iter().any(|(s, c)| s == "Cards" && c == "B4"),
        "Should find 'pineapple' in B4"
    );
    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_conditional_formatting_case_insensitive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_cards.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");
    sheet.get_cell_mut("A1").set_value("PINEAPPLE");
    sheet.get_cell_mut("A2").set_value("Pineapple");
    sheet.get_cell_mut("A3").set_value("pineapple");
    sheet.get_cell_mut("A4").set_value("PiNeApPlE");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-2".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = ConditionalFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let bold_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetBold { sheet: _, cell, bold } if *bold => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(bold_changes.len(), 4, "Should find all case variations");
    assert!(bold_changes.contains(&"A1".to_string()));
    assert!(bold_changes.contains(&"A2".to_string()));
    assert!(bold_changes.contains(&"A3".to_string()));
    assert!(bold_changes.contains(&"A4".to_string()));
}

#[test]
fn test_conditional_formatting_with_changed_range() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_cards.xlsx");

    tabula_cli_test_utils::create_cards_sheet_with_pineapple(&xlsx_path)
        .expect("Failed to create test spreadsheet");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-3".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: Some(ChangedRange {
            sheet: "Cards".to_string(),
            range: "A1:A5".to_string(),
        }),
    };

    let listener = ConditionalFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let bold_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetBold { sheet: _, cell, bold } if *bold => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(bold_changes.len(), 1, "Should only find A3 in range A1:A5");
    assert!(bold_changes.contains(&"A3".to_string()));
}

#[test]
fn test_conditional_formatting_ignores_non_string_cells() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_cards.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");
    sheet.get_cell_mut("A1").set_value("pineapple");
    sheet.get_cell_mut("A2").set_value_number(123);
    sheet.get_cell_mut("A3").set_value_bool(true);
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-4".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = ConditionalFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let bold_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetBold { sheet: _, cell, bold } if *bold => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(bold_changes.len(), 1, "Should only find string cell");
    assert!(bold_changes.contains(&"A1".to_string()));
}

#[test]
fn test_conditional_formatting_wrong_sheet() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_cards.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("OtherSheet");
    sheet.get_cell_mut("A1").set_value("pineapple");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-5".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = ConditionalFormattingListener;
    let result = listener.run(&snapshot, &context);

    assert!(result.is_err(), "Should fail when Cards sheet not found");
}
