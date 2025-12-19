use anyhow::{Context, Result};
use uuid::Uuid;

use crate::server::listener_runner::{Listener, ListenerContext, ListenerResult};
use crate::server::model::Change;
use crate::server::server_workbook_snapshot::{
    CellValue, SheetSnapshot, TableSnapshot, UsedRange, WorkbookSnapshot,
};

pub struct EnsureUuidListener;

impl Listener for EnsureUuidListener {
    fn name(&self) -> &str {
        "ensure_uuid"
    }

    fn run(
        &self,
        snapshot: &WorkbookSnapshot,
        context: &ListenerContext,
    ) -> Result<ListenerResult> {
        let mut changes = Vec::new();

        for table in &snapshot.tables {
            let sheet =
                snapshot.sheets.iter().find(|s| s.name == table.sheet_name).with_context(|| {
                    format!("Sheet '{}' not found for table '{}'", table.sheet_name, table.name)
                })?;

            let id_col_index = find_id_column(&table.columns)?;
            if id_col_index.is_none() {
                continue;
            }
            let id_col_index = id_col_index.unwrap();

            let rows_to_process = if let Some(ref changed_range) = context.changed_range {
                if changed_range.sheet == table.sheet_name {
                    let range = parse_a1_range(&changed_range.range)?;
                    if ranges_intersect(&range, &table.data_range) {
                        Some(get_intersecting_rows(&range, &table.data_range))
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            } else {
                None
            };

            process_table_rows(table, sheet, id_col_index, rows_to_process, &mut changes)?;
        }

        Ok(ListenerResult { changes, warnings: vec![] })
    }
}

fn find_id_column(columns: &[String]) -> Result<Option<usize>> {
    for (idx, col_name) in columns.iter().enumerate() {
        let normalized = normalize_column_name(col_name);
        if normalized.eq_ignore_ascii_case("id") {
            return Ok(Some(idx));
        }
    }
    Ok(None)
}

fn normalize_column_name(name: &str) -> String {
    name.replace('\u{00A0}', " ").trim().to_string()
}

fn process_table_rows(
    table: &TableSnapshot,
    sheet: &SheetSnapshot,
    id_col_index: usize,
    rows_to_process: Option<Vec<u32>>,
    changes: &mut Vec<Change>,
) -> Result<()> {
    let header_row = table.data_range.start_row;
    let data_start_row = header_row + 1;
    let data_end_row = table.data_range.end_row;

    let rows: Vec<u32> = if let Some(rows) = rows_to_process {
        rows.into_iter().map(|r| r - 1).collect()
    } else {
        (data_start_row..=data_end_row).collect()
    };

    for row_num in rows {
        if row_num < data_start_row || row_num > data_end_row {
            continue;
        }

        let excel_row = row_num + 1;
        let id_cell_ref =
            cell_ref_for_position(table.data_range.start_col + id_col_index as u32, excel_row);
        let id_value = normalize_blank(sheet.cell_values.get(&id_cell_ref));

        let has_other_content = table_row_has_other_content(table, sheet, row_num, id_col_index)?;

        if id_value.is_empty() && has_other_content {
            let uuid = Uuid::new_v4().to_string();
            changes.push(Change::SetValue {
                sheet: table.sheet_name.clone(),
                cell: id_cell_ref,
                value: uuid,
            });
        } else if !id_value.is_empty() && !has_other_content {
            changes.push(Change::ClearValue { sheet: table.sheet_name.clone(), cell: id_cell_ref });
        }
    }

    Ok(())
}

fn normalize_blank(value: Option<&CellValue>) -> String {
    match value {
        None | Some(CellValue::Empty) => String::new(),
        Some(CellValue::String(s)) => {
            let replaced = s.replace('\u{00A0}', " ");
            let normalized = replaced.trim();
            if normalized.is_empty() || normalized == "#ERROR" {
                String::new()
            } else {
                normalized.to_string()
            }
        }
        Some(CellValue::Float(f)) => {
            if f.is_finite() {
                f.to_string()
            } else {
                String::new()
            }
        }
        Some(CellValue::Int(i)) => i.to_string(),
        Some(CellValue::Bool(b)) => b.to_string(),
    }
}

fn table_row_has_other_content(
    table: &TableSnapshot,
    sheet: &SheetSnapshot,
    row_num: u32,
    id_col_index: usize,
) -> Result<bool> {
    for (col_idx, _) in table.columns.iter().enumerate() {
        if col_idx == id_col_index {
            continue;
        }

        let cell_ref =
            cell_ref_for_position(table.data_range.start_col + col_idx as u32, row_num + 1);
        if let Some(value) = sheet.cell_values.get(&cell_ref) {
            match value {
                CellValue::String(s) => {
                    let replaced = s.replace('\u{00A0}', " ");
                    let normalized = replaced.trim();
                    if !normalized.is_empty() && normalized != "#ERROR" {
                        return Ok(true);
                    }
                }
                CellValue::Float(f) => {
                    if f.is_finite() {
                        return Ok(true);
                    }
                }
                CellValue::Int(_) | CellValue::Bool(_) => return Ok(true),
                CellValue::Empty => {}
            }
        }
    }

    Ok(false)
}

fn cell_ref_for_position(col: u32, row: u32) -> String {
    format!("{}{}", col_index_to_letter(col), row)
}

fn col_index_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col + 1;
    while n > 0 {
        n -= 1;
        result.insert(0, char::from(b'A' + (n % 26) as u8));
        n /= 26;
    }
    result
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

fn ranges_intersect(range1: &CellRange, range2: &UsedRange) -> bool {
    range1.start_row <= range2.end_row
        && range1.end_row >= range2.start_row
        && range1.start_col <= range2.end_col
        && range1.end_col >= range2.start_col
}

fn get_intersecting_rows(changed_range: &CellRange, table_range: &UsedRange) -> Vec<u32> {
    let header_row = table_range.start_row;
    let data_start_row = header_row + 1;
    let data_start_excel_row = data_start_row + 1;
    let data_end_excel_row = table_range.end_row + 1;
    let start_excel_row = changed_range.start_row.max(data_start_excel_row);
    let end_excel_row = changed_range.end_row.min(data_end_excel_row);
    if start_excel_row > end_excel_row {
        Vec::new()
    } else {
        (start_excel_row..=end_excel_row).collect()
    }
}
