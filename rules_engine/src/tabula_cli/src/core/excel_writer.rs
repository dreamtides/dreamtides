use std::path::Path;

use anyhow::{Context, Result, bail};
use calamine::{self, Data, Range, Reader, Xlsx};

use crate::core::column_names;
use crate::core::excel_reader::ColumnType;

#[derive(Debug, Clone)]
pub struct ColumnLayout {
    pub name: String,
    pub normalized_name: String,
    pub column_type: ColumnType,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct TableLayout {
    pub name: String,
    pub normalized_name: String,
    pub sheet_name: String,
    pub data_start_row: u32,
    pub start_col: u32,
    pub data_rows: usize,
    pub columns: Vec<ColumnLayout>,
}

pub fn load_table_layouts(path: &Path) -> Result<Vec<TableLayout>> {
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;

    workbook
        .load_tables()
        .with_context(|| format!("Failed to load tables from {}", path.display()))?;

    let table_names: Vec<String> = workbook.table_names().iter().map(|s| s.to_string()).collect();
    if table_names.is_empty() {
        bail!(
            "No named Excel Tables found in {}. Tables are distinct from worksheets.",
            path.display()
        );
    }

    let mut tables = Vec::new();
    for table_name in &table_names {
        let table = workbook
            .table_by_name(table_name)
            .with_context(|| format!("Failed to read table '{table_name}'"))?;

        let sheet_name = table.sheet_name().to_string();
        let data = table.data();
        let (start_row_zero, start_col_zero) = data.start().unwrap_or((0, 0));
        let mut data_start_row = start_row_zero + 1;
        let start_col = start_col_zero + 1;
        let mut data_rows = data.height();
        let column_names: Vec<String> = table.columns().iter().map(|s| s.to_string()).collect();

        let formulas: Option<Range<String>> = workbook.worksheet_formula(&sheet_name).ok();

        let mut columns = Vec::new();
        for (idx, col_name) in column_names.iter().enumerate() {
            let abs_col = start_col_zero + idx as u32;
            let col_type = classify_column(data, formulas.as_ref(), idx, abs_col, start_row_zero);
            columns.push(ColumnLayout {
                name: col_name.clone(),
                normalized_name: column_names::normalize_column_name(col_name.as_str()),
                column_type: col_type,
                index: idx,
            });
        }

        let header_offset =
            if has_header_row(data, &column_names) && data_rows > 0 { 1 } else { 0 };
        data_rows = count_data_rows(data, header_offset);
        data_start_row += header_offset as u32;

        tables.push(TableLayout {
            name: table_name.clone(),
            normalized_name: column_names::normalize_table_name(table_name.as_str()),
            sheet_name,
            data_start_row,
            start_col,
            data_rows,
            columns,
        });
    }

    Ok(tables)
}

fn classify_column(
    data: &Range<Data>,
    formulas: Option<&Range<String>>,
    rel_col: usize,
    abs_col: u32,
    start_row: u32,
) -> ColumnType {
    let mut has_data = false;
    let mut has_formula = false;
    let mut is_image = false;

    if let Some(formula_range) = formulas {
        let (formula_start_row, formula_start_col) = formula_range.start().unwrap_or((0, 0));
        for row_idx in 0..data.height() {
            let abs_row = start_row as usize + row_idx;
            let abs_col_usize = abs_col as usize;
            let rel_row = abs_row as i64 - formula_start_row as i64;
            let rel_col = abs_col_usize as i64 - formula_start_col as i64;
            if rel_row < 0 || rel_col < 0 {
                continue;
            }
            if let Some(formula) = formula_range.get((rel_row as usize, rel_col as usize)) {
                if formula.is_empty() {
                    continue;
                }
                has_formula = true;
                if formula.to_uppercase().contains("IMAGE(") {
                    is_image = true;
                }
                break;
            }
        }
    }

    for row in data.rows() {
        if let Some(cell) = row.get(rel_col) {
            match cell {
                Data::Empty => {}
                Data::String(s) => {
                    let trimmed = s.trim_start();
                    if trimmed.starts_with('=') {
                        has_formula = true;
                        if trimmed.to_uppercase().contains("IMAGE(") {
                            is_image = true;
                        }
                        break;
                    }
                    has_data = true;
                }
                _ => {
                    has_data = true;
                }
            }
        }
    }

    if is_image {
        return ColumnType::Image;
    }

    if has_formula {
        return ColumnType::Formula;
    }

    if has_data { ColumnType::Data } else { ColumnType::Empty }
}

fn has_header_row(data: &Range<Data>, column_names: &[String]) -> bool {
    if column_names.is_empty() {
        return false;
    }
    let mut rows = data.rows();
    if let Some(first_row) = rows.next() {
        for (cell, col_name) in first_row.iter().zip(column_names.iter()) {
            match cell {
                Data::String(s) if s == col_name => {}
                _ => return false,
            }
        }
        return true;
    }
    false
}

fn count_data_rows(data: &Range<Data>, header_offset: usize) -> usize {
    let mut last_nonempty = None;
    for (idx, row) in data.rows().enumerate() {
        if idx < header_offset {
            continue;
        }
        if row_has_content(row) {
            last_nonempty = Some(idx - header_offset);
        }
    }
    last_nonempty.map(|i| i + 1).unwrap_or(0)
}

fn row_has_content(row: &[Data]) -> bool {
    for cell in row {
        match cell {
            Data::Empty => {}
            Data::String(s) if s.trim().is_empty() => {}
            _ => return true,
        }
    }
    false
}
