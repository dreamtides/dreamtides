use anyhow::{Context, Result};

use crate::server::listener_runner::{Listener, ListenerContext, ListenerResult};
use crate::server::model::Change;
use crate::server::server_workbook_snapshot::{CellValue, WorkbookSnapshot};
pub struct ConditionalFormattingListener;
impl Listener for ConditionalFormattingListener {
    fn name(&self) -> &str {
        "conditional_formatting"
    }

    fn run(
        &self,
        snapshot: &WorkbookSnapshot,
        context: &ListenerContext,
    ) -> Result<ListenerResult> {
        let cards_sheet =
            snapshot.sheets.iter().find(|s| s.name == "Cards").context("Cards sheet not found")?;
        let scan_range = if let Some(ref changed_range) = context.changed_range {
            if changed_range.sheet == "Cards" {
                Some(parse_a1_range(&changed_range.range)?)
            } else {
                None
            }
        } else {
            None
        };
        let mut changes = Vec::new();
        for (cell_ref, value) in &cards_sheet.cell_values {
            if let CellValue::String(text) = value
                && text.to_lowercase().contains("pineapple")
            {
                if let Some(ref range) = scan_range
                    && !cell_in_range(cell_ref, range)?
                {
                    continue;
                }
                changes.push(Change::SetBold {
                    sheet: "Cards".to_string(),
                    cell: cell_ref.clone(),
                    bold: true,
                });
            }
        }
        Ok(ListenerResult { changes, warnings: vec![] })
    }
}
struct CellRange {
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
}
fn parse_a1_range(range_str: &str) -> Result<CellRange> {
    let parts: Vec<&str> = range_str.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid range format: expected 'A1:B2', got '{}'", range_str);
    }
    let (start_row, start_col) = parse_cell_ref(parts[0])?;
    let (end_row, end_col) = parse_cell_ref(parts[1])?;
    Ok(CellRange {
        start_row: start_row.min(end_row),
        start_col: start_col.min(end_col),
        end_row: start_row.max(end_row),
        end_col: start_col.max(end_col),
    })
}
fn parse_cell_ref(cell_ref: &str) -> Result<(u32, u32)> {
    let mut col_letters = String::new();
    let mut row_digits = String::new();
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col_letters.push(ch.to_ascii_uppercase());
        } else if ch.is_ascii_digit() {
            row_digits.push(ch);
        } else if ch != '$' {
            anyhow::bail!("Invalid cell reference: '{}'", cell_ref);
        }
    }
    if col_letters.is_empty() || row_digits.is_empty() {
        anyhow::bail!("Invalid cell reference: '{}'", cell_ref);
    }
    let col = column_index(&col_letters)?;
    let row = row_digits.parse::<u32>().context("Invalid row number")?;
    Ok((row, col))
}
fn column_index(text: &str) -> Result<u32> {
    let mut value = 0u32;
    for ch in text.chars() {
        if !ch.is_ascii_uppercase() {
            anyhow::bail!("Invalid column '{}'", text);
        }
        value = value * 26 + (ch as u32 - 'A' as u32) + 1;
    }
    Ok(value)
}
fn cell_in_range(cell_ref: &str, range: &CellRange) -> Result<bool> {
    let (row, col) = parse_cell_ref(cell_ref)?;
    Ok(row >= range.start_row
        && row <= range.end_row
        && col >= range.start_col
        && col <= range.end_col)
}
