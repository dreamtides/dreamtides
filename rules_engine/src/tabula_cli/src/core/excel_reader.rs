use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use calamine::{self, Data, Range, Reader, Xlsx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Data,
    Formula,
    Image,
    Empty,
}

#[derive(Debug, Clone)]
pub enum CellValue {
    Empty,
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub column_type: ColumnType,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<BTreeMap<String, CellValue>>,
}

pub fn extract_tables(path: &Path) -> Result<Vec<TableInfo>> {
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
        let column_names: Vec<String> = table.columns().iter().map(|s| s.to_string()).collect();
        let data = table.data();
        let (start_row, start_col) = data.start().unwrap_or((0, 0));

        let formulas: Option<Range<String>> = workbook.worksheet_formula(&sheet_name).ok();

        let mut column_infos = Vec::new();
        for (col_idx, col_name) in column_names.iter().enumerate() {
            let abs_col = start_col + col_idx as u32;
            let col_type = classify_column(data, formulas.as_ref(), col_idx, abs_col, start_row);
            column_infos.push(ColumnInfo { name: col_name.clone(), column_type: col_type });
        }

        let mut rows = Vec::new();
        for row in data.rows() {
            let mut row_data = BTreeMap::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx >= column_infos.len() {
                    continue;
                }
                let col_info = &column_infos[col_idx];
                if col_info.column_type != ColumnType::Data {
                    continue;
                }
                let value = convert_cell(cell);
                if !matches!(value, CellValue::Empty) {
                    row_data.insert(col_info.name.clone(), value);
                }
            }
            if !row_data.is_empty() {
                rows.push(row_data);
            }
        }

        tables.push(TableInfo { name: table_name.clone(), columns: column_infos, rows });
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

fn convert_cell(cell: &Data) -> CellValue {
    match cell {
        Data::Empty => CellValue::Empty,
        Data::String(s) => CellValue::String(s.clone()),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                CellValue::Int(*f as i64)
            } else {
                CellValue::Float(*f)
            }
        }
        Data::Int(i) => CellValue::Int(*i),
        Data::Bool(b) => CellValue::Bool(*b),
        Data::DateTime(dt) => CellValue::Float(dt.as_f64()),
        Data::DateTimeIso(s) => CellValue::String(s.clone()),
        Data::DurationIso(s) => CellValue::String(s.clone()),
        Data::Error(e) => CellValue::String(format!("#{e:?}")),
    }
}
