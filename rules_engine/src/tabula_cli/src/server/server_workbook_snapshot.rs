use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use calamine::{Data, Reader, Xlsx};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[derive(Clone, Debug)]
pub struct WorkbookSnapshot {
    pub sheets: Vec<SheetSnapshot>,
    pub tables: Vec<TableSnapshot>,
}

#[derive(Clone, Debug)]
pub struct SheetSnapshot {
    pub name: String,
    pub used_range: Option<UsedRange>,
    pub cell_values: BTreeMap<String, CellValue>,
}

#[derive(Clone, Debug)]
pub struct UsedRange {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Clone, Debug)]
pub struct TableSnapshot {
    pub name: String,
    pub sheet_name: String,
    pub columns: Vec<String>,
    pub data_range: UsedRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Empty,
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub size: u64,
    pub mtime: i64,
}

pub fn read_snapshot(
    path: &Path,
    _expected_metadata: Option<FileMetadata>,
) -> Result<WorkbookSnapshot> {
    let mut last_error = None;
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64));
        }

        let metadata_before = capture_metadata(path)?;

        match read_workbook_internal(path) {
            Ok(snapshot) => {
                let metadata_after = capture_metadata(path)?;
                if metadata_before.size != metadata_after.size
                    || metadata_before.mtime != metadata_after.mtime
                {
                    if attempt < MAX_RETRIES {
                        last_error = Some(anyhow::anyhow!(
                            "Workbook changed during read (size: {}->{}, mtime: {}->{})",
                            metadata_before.size,
                            metadata_after.size,
                            metadata_before.mtime,
                            metadata_after.mtime
                        ));
                        continue;
                    }
                    return Err(anyhow::anyhow!(
                        "Workbook changed during read after {} retries",
                        MAX_RETRIES
                    ));
                }
                return Ok(snapshot);
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    last_error = Some(e);
                    continue;
                }
                return Err(last_error.unwrap_or(e));
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to read workbook")))
}

fn read_workbook_internal(path: &Path) -> Result<WorkbookSnapshot> {
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;

    let sheet_names: Vec<String> = workbook.sheet_names().clone();
    let mut sheets = Vec::new();

    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .with_context(|| format!("Failed to read sheet '{sheet_name}'"))?;

        let used_range = if range.is_empty() {
            None
        } else {
            let (start_row, start_col) = range.start().unwrap_or((0, 0));
            let (end_row, end_col) = range.end().unwrap_or((0, 0));
            Some(UsedRange { start_row, start_col, end_row, end_col })
        };

        let mut cell_values = BTreeMap::new();
        if let Some(ref used) = used_range {
            for (row_idx, row) in range.rows().enumerate() {
                let abs_row = used.start_row + row_idx as u32;
                for (col_idx, cell) in row.iter().enumerate() {
                    let abs_col = used.start_col + col_idx as u32;
                    let cell_ref = format!("{}{}", col_index_to_letter(abs_col), abs_row + 1);
                    let value = convert_cell(cell);
                    if !matches!(value, CellValue::Empty) {
                        cell_values.insert(cell_ref, value);
                    }
                }
            }
        }

        sheets.push(SheetSnapshot { name: sheet_name.clone(), used_range, cell_values });
    }

    let mut tables = Vec::new();
    if workbook.load_tables().is_ok() {
        let table_names: Vec<String> =
            workbook.table_names().iter().map(ToString::to_string).collect();
        for table_name in &table_names {
            if let Ok(table) = workbook.table_by_name(table_name) {
                let sheet_name = table.sheet_name().to_string();
                let columns: Vec<String> =
                    table.columns().iter().map(ToString::to_string).collect();
                let data = table.data();
                let (start_row, start_col) = data.start().unwrap_or((0, 0));
                let (end_row, end_col) = data.end().unwrap_or((0, 0));
                tables.push(TableSnapshot {
                    name: table_name.clone(),
                    sheet_name,
                    columns,
                    data_range: UsedRange { start_row, start_col, end_row, end_col },
                });
            }
        }
    }

    Ok(WorkbookSnapshot { sheets, tables })
}

fn capture_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Cannot read file metadata for {}", path.display()))?;
    let size = metadata.len();
    let mtime = metadata
        .modified()
        .or_else(|_| metadata.created())
        .unwrap_or_else(|_| SystemTime::now())
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    Ok(FileMetadata { size, mtime })
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
