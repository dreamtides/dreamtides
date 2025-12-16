use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use tempfile::Builder;
use toml::Value;
use umya_spreadsheet::reader::xlsx;
use umya_spreadsheet::writer;

use crate::core::excel_reader::ColumnType;
use crate::core::excel_writer::{ColumnLayout, TableLayout};
use crate::core::{column_names, excel_writer, paths};

#[derive(Clone, Debug)]
enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Clone, Debug)]
struct TomlRow {
    values: BTreeMap<String, (String, TomlValue)>,
}

#[derive(Clone, Debug)]
struct TomlTable {
    source_name: String,
    rows: Vec<TomlRow>,
}

#[derive(Clone, Debug)]
enum CellValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Empty,
}

#[derive(Clone, Debug)]
struct PreparedTable {
    layout: TableLayout,
    column_indices: Vec<u32>,
    rows: Vec<Vec<CellValue>>,
}

pub fn build_xls(
    dry_run: bool,
    toml_dir: Option<PathBuf>,
    xlsm_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
) -> Result<()> {
    let toml_dir = resolve_toml_dir(toml_dir)?;
    let template_path = resolve_xlsm_path(xlsm_path)?;
    if !template_path.exists() {
        bail!(
            "Original XLSM not found at {}. This file is required as a template.",
            template_path.display()
        );
    }
    let destination = resolve_output_path(&template_path, output_path)?;

    let layouts = excel_writer::load_table_layouts(&template_path)?;
    let toml_tables = load_toml_tables(&toml_dir)?;
    let prepared = prepare_tables(&layouts, &toml_tables, &toml_dir)?;

    if dry_run {
        return Ok(());
    }

    write_tables(&template_path, &destination, &prepared)
}

fn resolve_toml_dir(toml_dir: Option<PathBuf>) -> Result<PathBuf> {
    match toml_dir {
        Some(path) => Ok(path),
        None => paths::default_toml_dir(),
    }
}

fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

fn resolve_output_path(template_path: &Path, output_path: Option<PathBuf>) -> Result<PathBuf> {
    match output_path {
        Some(path) => Ok(path),
        None => Ok(template_path.to_path_buf()),
    }
}

fn load_toml_tables(dir: &Path) -> Result<BTreeMap<String, TomlTable>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Cannot open TOML directory {}", dir.display()))?;
    let mut tables = BTreeMap::new();

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("Cannot open TOML file {}", entry.path().display()))?;
        let value: Value = toml::from_str(&content)
            .with_context(|| format!("Cannot parse TOML file {}", entry.path().display()))?;
        let table = value.as_table().cloned().unwrap_or_default();
        for (key, data) in table {
            let normalized_name = column_names::normalize_table_name(key.as_str());
            if tables.contains_key(&normalized_name) {
                bail!("Unexpected TOML table '{normalized_name}' (not present in template)");
            }
            let rows = parse_toml_rows(key.as_str(), &data)?;
            tables.insert(normalized_name.clone(), TomlTable { source_name: key, rows });
        }
    }

    Ok(tables)
}

fn parse_toml_rows(table_name: &str, value: &Value) -> Result<Vec<TomlRow>> {
    match value {
        Value::Array(arr) => {
            if arr.iter().all(Value::is_table) {
                parse_table_rows(table_name, arr)
            } else {
                parse_single_column_rows(table_name, arr)
            }
        }
        _ => bail!("TOML file for table '{table_name}' must contain an array"),
    }
}

fn parse_table_rows(_table_name: &str, arr: &[Value]) -> Result<Vec<TomlRow>> {
    let mut rows = Vec::new();
    for (row_idx, row) in arr.iter().enumerate() {
        let table = row.as_table().cloned().ok_or_else(|| {
            anyhow!("Row {}: column '' value cannot be parsed: not a table", row_idx + 1)
        })?;
        let mut values = BTreeMap::new();
        for (key, val) in table {
            let normalized = column_names::normalize_column_name(key.as_str());
            if values.contains_key(&normalized) {
                bail!(
                    "Row {}: column '{}' value cannot be parsed: duplicate column",
                    row_idx + 1,
                    key
                );
            }
            let parsed = parse_scalar_value(row_idx + 1, key.as_str(), val)?;
            values.insert(normalized, (key, parsed));
        }
        rows.push(TomlRow { values });
    }
    Ok(rows)
}

fn parse_single_column_rows(table_name: &str, arr: &[Value]) -> Result<Vec<TomlRow>> {
    let mut rows = Vec::new();
    let column_name = column_names::normalize_table_name(table_name);
    for (row_idx, value) in arr.iter().enumerate() {
        let parsed = parse_scalar_value(row_idx + 1, column_name.as_str(), value.clone())?;
        let mut values = BTreeMap::new();
        values.insert(column_name.clone(), (column_name.clone(), parsed));
        rows.push(TomlRow { values });
    }
    Ok(rows)
}

fn parse_scalar_value(row_idx: usize, col: &str, value: Value) -> Result<TomlValue> {
    match value {
        Value::String(s) => Ok(TomlValue::String(s)),
        Value::Integer(i) => Ok(TomlValue::Integer(i)),
        Value::Float(f) => Ok(TomlValue::Float(f)),
        Value::Boolean(b) => Ok(TomlValue::Boolean(b)),
        _ => bail!("Row {row_idx}: column '{col}' value cannot be parsed: unsupported type"),
    }
}

fn prepare_tables(
    layouts: &[TableLayout],
    toml_tables: &BTreeMap<String, TomlTable>,
    toml_dir: &Path,
) -> Result<Vec<PreparedTable>> {
    let mut prepared = Vec::new();

    for (name, table) in toml_tables {
        if !layouts.iter().any(|layout| layout.normalized_name == *name) {
            bail!("Unexpected TOML table '{}' (not present in template)", table.source_name);
        }
    }

    for layout in layouts {
        let toml_table = toml_tables.get(&layout.normalized_name).ok_or_else(|| {
            anyhow::anyhow!(
                "TOML file for table '{}' not found at {}",
                layout.name,
                toml_dir.display()
            )
        })?;
        prepared.push(prepare_table(layout, toml_table)?);
    }

    Ok(prepared)
}

fn prepare_table(layout: &TableLayout, table: &TomlTable) -> Result<PreparedTable> {
    let data_columns: Vec<&ColumnLayout> =
        layout.columns.iter().filter(|c| matches!(c.column_type, ColumnType::Data)).collect();
    let mut column_map = BTreeMap::new();
    for col in &data_columns {
        column_map.insert(col.normalized_name.clone(), col);
    }

    if table.rows.len() != layout.data_rows {
        bail!(
            "Row count mismatch for '{}': TOML has {}, template has {}",
            layout.name,
            table.rows.len(),
            layout.data_rows
        );
    }

    let mut rows = Vec::new();
    for (row_idx, row) in table.rows.iter().enumerate() {
        for (normalized, (original, _)) in &row.values {
            if !column_map.contains_key(normalized) {
                bail!(
                    "Row {}: column '{}' does not match any writable column in '{}'",
                    row_idx + 1,
                    original,
                    layout.name
                );
            }
        }

        let mut prepared_row = Vec::new();
        for col in &data_columns {
            let value = row
                .values
                .get(&col.normalized_name)
                .map(|(_, v)| v.clone())
                .map(cell_from_toml)
                .unwrap_or(CellValue::Empty);
            prepared_row.push(value);
        }
        rows.push(prepared_row);
    }

    let column_indices: Vec<u32> =
        data_columns.iter().map(|col| layout.start_col + col.index as u32).collect();

    Ok(PreparedTable { layout: layout.clone(), column_indices, rows })
}

fn cell_from_toml(value: TomlValue) -> CellValue {
    match value {
        TomlValue::String(s) => CellValue::String(s),
        TomlValue::Integer(i) => CellValue::Integer(i),
        TomlValue::Float(f) => CellValue::Float(f),
        TomlValue::Boolean(b) => CellValue::Boolean(b),
    }
}

fn write_tables(template_path: &Path, destination: &Path, tables: &[PreparedTable]) -> Result<()> {
    let mut book = xlsx::read(template_path)
        .with_context(|| format!("Cannot open spreadsheet at {}", template_path.display()))?;

    for table in tables {
        let sheet =
            book.get_sheet_by_name_mut(table.layout.sheet_name.as_str()).ok_or_else(|| {
                anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name)
            })?;
        for (row_idx, row) in table.rows.iter().enumerate() {
            let row_num = table.layout.data_start_row + row_idx as u32;
            for (col_idx, value) in row.iter().enumerate() {
                let col_num = table.column_indices[col_idx];
                let cell = sheet.get_cell_mut((col_num, row_num));
                match value {
                    CellValue::String(s) => cell.set_value(s),
                    CellValue::Integer(i) => cell.set_value_number(*i as f64),
                    CellValue::Float(f) => cell.set_value_number(*f),
                    CellValue::Boolean(b) => cell.set_value_bool(*b),
                    CellValue::Empty => cell.set_value(""),
                };
            }
        }
    }

    let parent = destination.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let suffix = destination
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_else(|| ".xlsx".to_string());
    let temp = Builder::new()
        .prefix("tabula_build_xls")
        .suffix(suffix.as_str())
        .tempfile_in(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let temp_path = temp.into_temp_path();
    let temp_buf = temp_path.to_path_buf();
    writer::xlsx::write(&book, &temp_buf)?;
    temp_path
        .persist(destination)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;

    Ok(())
}
