use tabula_cli::server::listener_runner::{Listener, ListenerContext};
use tabula_cli::server::listeners::boxicons::BoxiconsListener;
use tabula_cli::server::listeners::conditional_formatting::ConditionalFormattingListener;
use tabula_cli::server::listeners::fluent_rules_text::FluentRulesTextListener;
use tabula_cli::server::listeners::partial_formatting::PartialFormattingListener;
use tabula_cli::server::model::{Change, ChangedRange};
use tabula_cli::server::server_workbook_snapshot::read_snapshot;
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use umya_spreadsheet::writer::xlsx;

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
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

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
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

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

#[test]
fn test_partial_formatting_finds_jackalope() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("The jackalope is here");
    sheet.get_cell_mut("A2").set_value("No match");
    sheet.get_cell_mut("A3").set_value("JACKALOPE in uppercase");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-1".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet, cell, rgb, spans } => {
                Some((sheet.clone(), cell.clone(), rgb.clone(), spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 2, "Expected 2 cells with jackalope");
    assert!(
        span_changes.iter().any(|(s, c, rgb, spans)| {
            s == "Sheet1"
                && c == "A1"
                && rgb == "FFA500"
                && spans.len() == 1
                && spans[0].start == 5
                && spans[0].length == 9
        }),
        "Should find 'jackalope' starting at position 5 in A1"
    );
    assert!(
        span_changes.iter().any(|(s, c, rgb, spans)| {
            s == "Sheet1"
                && c == "A3"
                && rgb == "FFA500"
                && spans.len() == 1
                && spans[0].start == 1
                && spans[0].length == 9
        }),
        "Should find 'JACKALOPE' starting at position 1 in A3"
    );
    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_partial_formatting_case_insensitive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("JACKALOPE");
    sheet.get_cell_mut("A2").set_value("Jackalope");
    sheet.get_cell_mut("A3").set_value("jackalope");
    sheet.get_cell_mut("A4").set_value("JaCkAlOpE");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-2".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet: _, cell, .. } => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 4, "Should find all case variations");
    assert!(span_changes.contains(&"A1".to_string()));
    assert!(span_changes.contains(&"A2".to_string()));
    assert!(span_changes.contains(&"A3".to_string()));
    assert!(span_changes.contains(&"A4".to_string()));
}

#[test]
fn test_partial_formatting_multiple_occurrences() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("jackalope and jackalope again");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-3".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet: _, cell, spans, .. } if cell == "A1" => {
                Some(spans.clone())
            }
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 1, "Should have one change for A1");
    let spans = &span_changes[0];
    assert_eq!(spans.len(), 2, "Should find 2 non-overlapping occurrences");
    assert_eq!(spans[0].start, 1, "First occurrence starts at position 1");
    assert_eq!(spans[0].length, 9, "First occurrence length is 9");
    assert_eq!(spans[1].start, 15, "Second occurrence starts at position 15");
    assert_eq!(spans[1].length, 9, "Second occurrence length is 9");
}

#[test]
fn test_partial_formatting_overlapping_occurrences() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("jackalopealope");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-4".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet: _, cell, spans, .. } if cell == "A1" => {
                Some(spans.clone())
            }
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 1, "Should have one change for A1");
    let spans = &span_changes[0];
    assert_eq!(spans.len(), 1, "Should find only 1 non-overlapping occurrence");
    assert_eq!(spans[0].start, 1, "Occurrence starts at position 1");
    assert_eq!(spans[0].length, 9, "Occurrence length is 9");
}

#[test]
fn test_partial_formatting_with_changed_range() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("jackalope here");
    sheet.get_cell_mut("A2").set_value("no match");
    sheet.get_cell_mut("B1").set_value("another jackalope");
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-5".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: Some(ChangedRange {
            sheet: "Sheet1".to_string(),
            range: "A1:A2".to_string(),
        }),
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet: _, cell, .. } => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 1, "Should only find A1 in range A1:A2");
    assert!(span_changes.contains(&"A1".to_string()));
}

#[test]
fn test_partial_formatting_ignores_non_string_cells() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("jackalope");
    sheet.get_cell_mut("A2").set_value_number(123);
    sheet.get_cell_mut("A3").set_value_bool(true);
    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-6".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet: _, cell, .. } => Some(cell.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 1, "Should only find string cell");
    assert!(span_changes.contains(&"A1".to_string()));
}

#[test]
fn test_partial_formatting_multiple_sheets() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_jackalope.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet1 = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet1.set_name("Sheet1");
    sheet1.get_cell_mut("A1").set_value("jackalope in sheet1");

    let _ = book.new_sheet("Sheet2");
    let sheet2_index = 1;
    let sheet2 = book.get_sheet_mut(&sheet2_index).expect("Sheet2 should exist");
    sheet2.get_cell_mut("A1").set_value("jackalope in sheet2");

    umya_spreadsheet::writer::xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-jack-7".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = PartialFormattingListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let span_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontColorSpans { sheet, cell, .. } => Some((sheet.clone(), cell.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(span_changes.len(), 2, "Should find jackalope in both sheets");
    assert!(span_changes.contains(&("Sheet1".to_string(), "A1".to_string())));
    assert!(span_changes.contains(&("Sheet2".to_string(), "A1".to_string())));
}

#[test]
fn test_boxicons_replaces_x_with_icon() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_boxicons.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("Click {x} to expand");
    sheet.get_cell_mut("A2").set_value("{x}");
    sheet.get_cell_mut("A3").set_value("No icon here");
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-boxicons-1".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = BoxiconsListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let set_value_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetValue { cell, value, .. } => Some((cell.clone(), value.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(set_value_changes.len(), 2, "Should create formatted versions in 2 cells");

    let b1_value =
        set_value_changes.iter().find(|(cell, _)| cell == "B1").map(|(_, value)| value.clone());
    assert_eq!(b1_value, Some("Click \u{200E}\u{FEFC} to expand".to_string()));

    let b2_value =
        set_value_changes.iter().find(|(cell, _)| cell == "B2").map(|(_, value)| value.clone());
    assert_eq!(b2_value, Some("\u{200E}\u{FEFC}".to_string()));

    let font_name_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontNameSpans { cell, font_name, spans, .. } => {
                Some((cell.clone(), font_name.clone(), spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(font_name_changes.len(), 2);

    let b1_font = font_name_changes.iter().find(|(cell, _, _)| cell == "B1");
    assert!(b1_font.is_some());
    let (_, font_name, spans) = b1_font.unwrap();
    assert_eq!(font_name, "boxicons");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 7);
    assert_eq!(spans[0].length, 2);

    let font_size_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontSizeSpans { cell, points, spans, .. } => {
                Some((cell.clone(), *points, spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(font_size_changes.len(), 2);

    let b1_size = font_size_changes.iter().find(|(cell, _, _)| cell == "B1");
    assert!(b1_size.is_some());

    let subscript_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetSubscriptSpans { cell, subscript, spans, .. } => {
                Some((cell.clone(), *subscript, spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(subscript_changes.len(), 2);

    let b1_subscript = subscript_changes.iter().find(|(cell, _, _)| cell == "B1");
    assert!(b1_subscript.is_some());
    let (_, is_subscript, spans) = b1_subscript.unwrap();
    assert!(is_subscript);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 7);
    assert_eq!(spans[0].length, 2);

    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_boxicons_multiple_occurrences() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_boxicons_multi.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("{x} and {x}");
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-boxicons-2".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = BoxiconsListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let set_value_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetValue { cell, value, .. } => Some((cell.clone(), value.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(set_value_changes.len(), 1);
    let b1_value = set_value_changes.iter().find(|(cell, _)| cell == "B1");
    assert_eq!(
        b1_value,
        Some(&("B1".to_string(), "\u{200E}\u{FEFC} and \u{200E}\u{FEFC}".to_string()))
    );

    let font_name_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontNameSpans { cell, spans, .. } if cell == "B1" => Some(spans.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(font_name_changes.len(), 1);
    let spans = &font_name_changes[0];
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start, 1);
    assert_eq!(spans[0].length, 2);
    assert_eq!(spans[1].start, 8);
    assert_eq!(spans[1].length, 2);

    let subscript_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetSubscriptSpans { cell, subscript, spans, .. } if cell == "B1" => {
                Some((*subscript, spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(subscript_changes.len(), 1);
    let (is_subscript, subscript_spans) = &subscript_changes[0];
    assert!(is_subscript);
    assert_eq!(subscript_spans.len(), 2);
    assert_eq!(subscript_spans[0].start, 1);
    assert_eq!(subscript_spans[0].length, 2);
    assert_eq!(subscript_spans[1].start, 8);
    assert_eq!(subscript_spans[1].length, 2);
}

#[test]
fn test_boxicons_no_match() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_boxicons_none.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("No icons here");
    sheet.get_cell_mut("A2").set_value("{ x }");
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-boxicons-3".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = BoxiconsListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    assert_eq!(result.changes.len(), 0, "Should not generate any changes");
    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_boxicons_multiple_icon_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_boxicons_types.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Sheet1");
    sheet.get_cell_mut("A1").set_value("{e}");
    sheet.get_cell_mut("A2").set_value("{fast}");
    sheet.get_cell_mut("A3").set_value("{p}");
    sheet.get_cell_mut("A4").set_value("{x} {e} {fast} {p}");
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-boxicons-4".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = BoxiconsListener;
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let set_value_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetValue { cell, value, .. } => Some((cell.clone(), value.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(set_value_changes.len(), 4);

    let b1_value = set_value_changes.iter().find(|(cell, _)| cell == "B1");
    assert_eq!(b1_value, Some(&("B1".to_string(), "\u{200E}\u{F407}".to_string())));

    let b2_value = set_value_changes.iter().find(|(cell, _)| cell == "B2");
    assert_eq!(b2_value, Some(&("B2".to_string(), "\u{200E}\u{F93A}".to_string())));

    let b3_value = set_value_changes.iter().find(|(cell, _)| cell == "B3");
    assert_eq!(b3_value, Some(&("B3".to_string(), "\u{200E}\u{FC6A}".to_string())));

    let b4_value = set_value_changes.iter().find(|(cell, _)| cell == "B4");
    assert_eq!(
        b4_value,
        Some(&(
            "B4".to_string(),
            "\u{200E}\u{FEFC} \u{200E}\u{F407} \u{200E}\u{F93A} \u{200E}\u{FC6A}".to_string()
        ))
    );

    let font_name_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetFontNameSpans { cell, spans, .. } if cell == "B4" => Some(spans.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(font_name_changes.len(), 1);
    let spans = &font_name_changes[0];
    assert_eq!(spans.len(), 4);
    assert_eq!(spans[0].start, 1);
    assert_eq!(spans[0].length, 2);
    assert_eq!(spans[1].start, 4);
    assert_eq!(spans[1].length, 2);
    assert_eq!(spans[2].start, 7);
    assert_eq!(spans[2].length, 2);
    assert_eq!(spans[3].start, 10);
    assert_eq!(spans[3].length, 2);

    let subscript_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetSubscriptSpans { cell, subscript, spans, .. } if cell == "B4" => {
                Some((*subscript, spans.clone()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(subscript_changes.len(), 1);
    let (is_subscript, subscript_spans) = &subscript_changes[0];
    assert!(is_subscript);
    assert_eq!(subscript_spans.len(), 4);
}

#[test]
fn test_fluent_rules_text_formats_expression() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_fluent.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");

    sheet.get_cell_mut("A1").set_value("RulesText3");
    sheet.get_cell_mut("B1").set_value("Fluent Output");
    sheet.get_cell_mut("A2").set_value(
        "{ -pay-energy(n: 2) }: { -draw-cards(n: 3) } { -this-character-gains-spark(delta: 1) }",
    );

    let mut table = umya_spreadsheet::structs::Table::default();
    table.set_name("Cards2");
    table.set_display_name("Cards2");
    table.set_area(("A1", "B2"));
    table.add_column(tabula_cli_test_utils::make_column("RulesText3"));
    table.add_column(tabula_cli_test_utils::make_column("Fluent Output"));
    sheet.add_table(table);

    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-fluent-1".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = FluentRulesTextListener::new().expect("Failed to create listener");
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let set_value_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetValue { cell, value, .. } => Some((cell.clone(), value.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(set_value_changes.len(), 1, "Should create formatted output in 1 cell");

    let b2_value = set_value_changes.iter().find(|(cell, _)| cell == "B2");
    assert_eq!(
        b2_value,
        Some(&(
            "B2".to_string(),
            "Pay 2 energy: Draw 3 cards. This character gains 1 spark.".to_string()
        ))
    );

    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_fluent_rules_text_multiple_rows() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_fluent_multi.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");

    sheet.get_cell_mut("A1").set_value("RulesText3");
    sheet.get_cell_mut("B1").set_value("Fluent Output");
    sheet.get_cell_mut("A2").set_value("{ -pay-energy(n: 1) }");
    sheet.get_cell_mut("A3").set_value("{ -draw-cards(n: 2) }");
    sheet.get_cell_mut("A4").set_value("");

    let mut table = umya_spreadsheet::structs::Table::default();
    table.set_name("Cards2");
    table.set_display_name("Cards2");
    table.set_area(("A1", "B4"));
    table.add_column(tabula_cli_test_utils::make_column("RulesText3"));
    table.add_column(tabula_cli_test_utils::make_column("Fluent Output"));
    sheet.add_table(table);

    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-fluent-2".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = FluentRulesTextListener::new().expect("Failed to create listener");
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    let set_value_changes: Vec<_> = result
        .changes
        .iter()
        .filter_map(|c| match c {
            Change::SetValue { cell, value, .. } => Some((cell.clone(), value.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(set_value_changes.len(), 2, "Should format 2 non-empty cells");

    let b2_value = set_value_changes.iter().find(|(cell, _)| cell == "B2");
    assert_eq!(b2_value, Some(&("B2".to_string(), "Pay 1 energy".to_string())));

    let b3_value = set_value_changes.iter().find(|(cell, _)| cell == "B3");
    assert_eq!(b3_value, Some(&("B3".to_string(), "Draw 2 cards.".to_string())));

    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_fluent_rules_text_no_cards_table() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_fluent_no_table.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("OtherSheet");
    sheet.get_cell_mut("A1").set_value("No table here");
    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-fluent-3".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = FluentRulesTextListener::new().expect("Failed to create listener");
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    assert_eq!(result.changes.len(), 0, "Should not generate any changes without Cards table");
    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_fluent_rules_text_no_rules_text_column() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_fluent_no_column.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");

    sheet.get_cell_mut("A1").set_value("Name");
    sheet.get_cell_mut("B1").set_value("Cost");
    sheet.get_cell_mut("A2").set_value("Card 1");
    sheet.get_cell_mut("B2").set_value_number(3);

    let mut table = umya_spreadsheet::structs::Table::default();
    table.set_name("Cards");
    table.set_display_name("Cards");
    table.set_area(("A1", "B2"));
    table.add_column(tabula_cli_test_utils::make_column("Name"));
    table.add_column(tabula_cli_test_utils::make_column("Cost"));
    sheet.add_table(table);

    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-fluent-4".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = FluentRulesTextListener::new().expect("Failed to create listener");
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    assert_eq!(
        result.changes.len(),
        0,
        "Should not generate any changes without Rules Text column"
    );
    assert!(result.warnings.is_empty(), "Should have no warnings");
}

#[test]
fn test_fluent_rules_text_invalid_expression() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let xlsx_path = temp_dir.path().join("test_fluent_invalid.xlsx");

    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("Cards");

    sheet.get_cell_mut("A1").set_value("RulesText3");
    sheet.get_cell_mut("B1").set_value("Fluent Output");
    sheet.get_cell_mut("A2").set_value("{ unclosed bracket");

    let mut table = umya_spreadsheet::structs::Table::default();
    table.set_name("Cards2");
    table.set_display_name("Cards2");
    table.set_area(("A1", "B2"));
    table.add_column(tabula_cli_test_utils::make_column("RulesText3"));
    table.add_column(tabula_cli_test_utils::make_column("Fluent Output"));
    sheet.add_table(table);

    xlsx::write(&book, &xlsx_path).expect("Write workbook");

    let snapshot = read_snapshot(&xlsx_path, None).expect("Failed to read snapshot");

    let context = ListenerContext {
        request_id: "test-fluent-5".to_string(),
        workbook_path: xlsx_path.to_string_lossy().to_string(),
        changed_range: None,
    };

    let listener = FluentRulesTextListener::new().expect("Failed to create listener");
    let result = listener.run(&snapshot, &context).expect("Listener should succeed");

    assert_eq!(result.changes.len(), 0, "Should not generate changes for invalid expression");
    assert_eq!(result.warnings.len(), 1, "Should have exactly one warning");
    assert!(
        result.warnings[0].contains("Failed to format cell A2"),
        "Warning should mention the problematic cell"
    );
}
