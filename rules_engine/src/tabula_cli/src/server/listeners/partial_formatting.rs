use anyhow::{Context, Result};

use crate::server::listener_runner::{Listener, ListenerContext, ListenerResult};
use crate::server::model::{Change, Span};
use crate::server::server_workbook_snapshot::{CellValue, WorkbookSnapshot};
pub struct PartialFormattingListener;
impl Listener for PartialFormattingListener {
    fn name(&self) -> &str {
        "partial_formatting"
    }

    fn run(
        &self,
        snapshot: &WorkbookSnapshot,
        context: &ListenerContext,
    ) -> Result<ListenerResult> {
        const COLOR_ORANGE: &str = "FFA500";
        let mut changes = Vec::new();
        for sheet in &snapshot.sheets {
            let cells_to_scan = if let Some(ref changed_range) = context.changed_range {
                if changed_range.sheet == sheet.name {
                    let range = parse_a1_range(&changed_range.range)?;
                    sheet
                        .cell_values
                        .iter()
                        .filter(|(cell_ref, _)| cell_in_range(cell_ref, &range).unwrap_or(false))
                        .collect::<Vec<_>>()
                } else {
                    continue;
                }
            } else {
                sheet.cell_values.iter().collect::<Vec<_>>()
            };
            for (cell_ref, value) in cells_to_scan {
                if let CellValue::String(text) = value {
                    let spans = find_jackalope_spans(text);
                    if !spans.is_empty() {
                        changes.push(Change::SetFontColorSpans {
                            sheet: sheet.name.clone(),
                            cell: cell_ref.clone(),
                            rgb: COLOR_ORANGE.to_string(),
                            spans,
                        });
                    }
                }
            }
        }
        Ok(ListenerResult { changes, warnings: vec![] })
    }
}
fn find_jackalope_spans(text: &str) -> Vec<Span> {
    const JACKALOPE: &str = "jackalope";
    let text_lower = text.to_lowercase();
    let pattern_lower = JACKALOPE.to_lowercase();
    let pattern_len = pattern_lower.len();
    let mut spans = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = text_lower[search_start..].find(&pattern_lower) {
        let absolute_pos = search_start + pos;
        let start = (absolute_pos + 1) as u32;
        let length = pattern_len as u32;
        let overlaps = spans.iter().any(|s: &Span| {
            let s_end = s.start + s.length - 1;
            let new_end = start + length - 1;
            (start <= s_end && start >= s.start) || (new_end >= s.start && new_end <= s_end)
        });
        if !overlaps {
            spans.push(Span { start, length });
        }
        search_start = absolute_pos + pattern_len;
    }
    spans
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
