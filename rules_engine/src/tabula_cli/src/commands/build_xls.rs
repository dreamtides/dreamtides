use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use roxmltree::Document;
use tempfile::Builder;
use toml::Value;
use umya_spreadsheet::helper::coordinate::string_from_column_index;
use umya_spreadsheet::reader::xlsx;
use umya_spreadsheet::structs::Worksheet;
use umya_spreadsheet::{Spreadsheet, writer};
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::core::excel_reader::ColumnType;
use crate::core::excel_writer::{ColumnLayout, TableLayout};
use crate::core::{column_names, excel_writer, paths, toml_data};

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

#[derive(Clone)]
struct ZipRecord {
    data: Vec<u8>,
    compression: CompressionMethod,
    modified: DateTime,
    is_dir: bool,
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
    output_path.ok_or_else(|| {
        anyhow!(
            "--output-path is required (pass the XLSM template path to overwrite in place; template: {})",
            template_path.display()
        )
    })
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
        let value = toml_data::canonicalize_numbers(value);
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

fn copy_row(sheet: &mut Worksheet, source_row: u32, target_row: u32, layout: &TableLayout) {
    for column in &layout.columns {
        let col_num = layout.start_col + column.index as u32;
        if let Some(source_cell) = sheet.get_cell((col_num, source_row)) {
            let value = source_cell.get_cell_value().clone();
            let style = source_cell.get_style().clone();
            let target_cell = sheet.get_cell_mut((col_num, target_row));
            target_cell.set_cell_value(value);
            target_cell.set_style(style);
        }
    }
}

fn adjusted_row(row: u32, adjustments: &[(u32, i32)]) -> u32 {
    let mut value = row as i64;
    for (position, delta) in adjustments {
        if value >= *position as i64 {
            value = (value + *delta as i64).max(1);
        }
    }
    value as u32
}

fn write_single_table(table: &PreparedTable, book: &mut Spreadsheet) -> Result<()> {
    let sheet = book.get_sheet_by_name_mut(table.layout.sheet_name.as_str()).ok_or_else(|| {
        anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name)
    })?;
    // umya's insert_new_row re-parses every formula on the sheet and can hang on
    // large single-table sheets; bypass it here
    let (header_row, start_col_index, end_col, area_end_row) = {
        let table_def = sheet
            .get_tables_mut()
            .iter_mut()
            .find(|t| t.get_name() == table.layout.name)
            .ok_or_else(|| {
                anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name)
            })?;
        let area = table_def.get_area().clone();
        (*area.0.get_row_num(), *area.0.get_col_num(), *area.1.get_col_num(), *area.1.get_row_num())
    };
    let target_end_row = std::cmp::max(area_end_row, header_row + table.rows.len() as u32);
    let source_row = if table.layout.data_rows > 0 {
        table.layout.data_start_row + table.layout.data_rows as u32 - 1
    } else {
        table.layout.data_start_row
    };
    let max_existing_row = target_end_row;

    for (row_idx, row) in table.rows.iter().enumerate() {
        let row_num = table.layout.data_start_row + row_idx as u32;
        if row_num > source_row {
            copy_row(sheet, source_row, row_num, &table.layout);
        }
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

    let clear_start = table.layout.data_start_row + table.rows.len() as u32;
    if clear_start <= max_existing_row {
        for row_num in clear_start..=max_existing_row {
            for &col_num in &table.column_indices {
                let cell = sheet.get_cell_mut((col_num, row_num));
                cell.set_value("");
                cell.get_cell_value_mut().remove_formula();
            }
        }
    }

    let start_cell = format!("{}{}", string_from_column_index(&start_col_index), header_row);
    let end_cell = format!("{}{}", string_from_column_index(&end_col), target_end_row);
    let table_def =
        sheet.get_tables_mut().iter_mut().find(|t| t.get_name() == table.layout.name).ok_or_else(
            || anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name),
        )?;
    table_def.set_area((start_cell.as_str(), end_cell.as_str()));

    Ok(())
}

fn write_tables(template_path: &Path, destination: &Path, tables: &[PreparedTable]) -> Result<()> {
    let mut book = xlsx::read(template_path)
        .with_context(|| format!("Cannot open spreadsheet at {}", template_path.display()))?;

    let mut table_refs: Vec<&PreparedTable> = tables.iter().collect();
    table_refs.sort_by(|a, b| {
        let sheet_cmp = a.layout.sheet_name.cmp(&b.layout.sheet_name);
        if sheet_cmp == Ordering::Equal {
            a.layout.data_start_row.cmp(&b.layout.data_start_row)
        } else {
            sheet_cmp
        }
    });

    let mut sheet_column_ranges: BTreeMap<String, Vec<(String, u32, u32)>> = BTreeMap::new();
    for table in tables {
        let end_col = table.layout.start_col + table.layout.columns.len() as u32 - 1;
        sheet_column_ranges.entry(table.layout.sheet_name.clone()).or_default().push((
            table.layout.name.clone(),
            table.layout.start_col,
            end_col,
        ));
    }

    let mut overlapping_columns: HashSet<String> = HashSet::new();
    for ranges in sheet_column_ranges.values() {
        for i in 0..ranges.len() {
            for j in (i + 1)..ranges.len() {
                let (name_a, start_a, end_a) = &ranges[i];
                let (name_b, start_b, end_b) = &ranges[j];
                let cols_overlap = !(end_a < start_b || end_b < start_a);
                if cols_overlap {
                    overlapping_columns.insert(name_a.clone());
                    overlapping_columns.insert(name_b.clone());
                }
            }
        }
    }

    let mut sheet_table_counts: BTreeMap<String, usize> = BTreeMap::new();
    for table in &table_refs {
        let entry = sheet_table_counts.entry(table.layout.sheet_name.clone()).or_insert(0);
        *entry += 1;
    }

    let mut sheet_adjustments: BTreeMap<String, Vec<(u32, i32)>> = BTreeMap::new();

    for table in table_refs {
        let single_table_sheet =
            *sheet_table_counts.get(table.layout.sheet_name.as_str()).unwrap_or(&0) == 1;
        if single_table_sheet {
            write_single_table(table, &mut book)?;
            continue;
        }
        let has_column_overlap = overlapping_columns.contains(&table.layout.name);
        let adjustments = sheet_adjustments.entry(table.layout.sheet_name.clone()).or_default();
        let start_row = adjusted_row(table.layout.data_start_row, adjustments);
        let current_rows = table.layout.data_rows as i32;
        let desired_rows = table.rows.len() as i32;
        let diff = desired_rows - current_rows;

        let sheet =
            book.get_sheet_by_name_mut(table.layout.sheet_name.as_str()).ok_or_else(|| {
                anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name)
            })?;

        let table_index =
            sheet.get_tables().iter().position(|t| t.get_name() == table.layout.name).ok_or_else(
                || anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name),
            )?;
        let (header_row, start_col_index, end_col, totals_rows, area_end_row) = {
            let table_def = &sheet.get_tables()[table_index];
            let area = table_def.get_area();
            let totals = if *table_def.get_totals_row_shown() {
                std::cmp::max(1, *table_def.get_totals_row_count())
            } else {
                *table_def.get_totals_row_count()
            };
            (
                *area.0.get_row_num(),
                *area.0.get_col_num(),
                *area.1.get_col_num(),
                totals,
                *area.1.get_row_num(),
            )
        };
        let header_row = adjusted_row(header_row, adjustments);

        let source_row = if table.layout.data_rows > 0 {
            start_row + table.layout.data_rows as u32 - 1
        } else {
            start_row
        };

        if diff > 0 && has_column_overlap {
            let insert_at = start_row + table.layout.data_rows as u32;
            sheet.insert_new_row(&insert_at, &(diff as u32));
            for i in 0..diff {
                let target_row = insert_at + i as u32;
                copy_row(sheet, source_row, target_row, &table.layout);
            }
            adjustments.push((insert_at, diff));
        } else if diff < 0 && has_column_overlap {
            let remove_start = start_row + desired_rows as u32;
            let remove_count = (-diff) as u32;
            sheet.remove_row(&remove_start, &remove_count);
            adjustments.push((remove_start, diff));
        }

        for (row_idx, row) in table.rows.iter().enumerate() {
            let row_num = start_row + row_idx as u32;
            if row_num > source_row {
                copy_row(sheet, source_row, row_num, &table.layout);
            }
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

        let target_end_row = header_row + table.rows.len() as u32 + totals_rows;
        let clear_start = start_row + table.rows.len() as u32;
        let max_existing_row =
            std::cmp::max(target_end_row, adjusted_row(area_end_row, adjustments));
        if clear_start <= max_existing_row {
            for row_num in clear_start..=max_existing_row {
                for &col_num in &table.column_indices {
                    let cell = sheet.get_cell_mut((col_num, row_num));
                    cell.set_value("");
                    cell.get_cell_value_mut().remove_formula();
                }
            }
        }

        let start_cell = format!("{}{}", string_from_column_index(&start_col_index), header_row);
        let end_cell = format!("{}{}", string_from_column_index(&end_col), target_end_row);
        let table_def = sheet.get_tables_mut().get_mut(table_index).ok_or_else(|| {
            anyhow::anyhow!("Table '{}' not found in original XLSM", table.layout.name)
        })?;
        table_def.set_area((start_cell.as_str(), end_cell.as_str()));
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
    merge_template_parts(template_path, &temp_buf)?;
    ensure_recalc_on_open(&temp_buf)?;
    temp_path
        .persist(destination)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;

    Ok(())
}

fn merge_template_parts(template_path: &Path, updated_path: &Path) -> Result<()> {
    let (mut template_records, template_order) = read_zip_records(template_path)?;
    let (updated_records, updated_order) = read_zip_records(updated_path)?;
    let mut file_order = template_order;
    for name in updated_order {
        if !file_order.contains(&name) {
            file_order.push(name);
        }
    }

    for (name, record) in &updated_records {
        if should_replace_from_updated(name.as_str(), &template_records) {
            template_records.insert(name.clone(), record.clone());
        }
    }

    let content_types = merged_content_types(
        template_records
            .get("[Content_Types].xml")
            .context("Spreadsheet is missing [Content_Types].xml")?
            .data
            .as_slice(),
        updated_records
            .get("[Content_Types].xml")
            .context("Updated spreadsheet is missing [Content_Types].xml")?
            .data
            .as_slice(),
    )?;
    if let Some(record) = template_records.get_mut("[Content_Types].xml") {
        record.data = content_types;
    }

    write_zip_records(updated_path, template_records, &file_order)
}

fn should_replace_from_updated(name: &str, template_records: &BTreeMap<String, ZipRecord>) -> bool {
    if !template_records.contains_key(name) {
        return true;
    }
    name.starts_with("xl/worksheets/")
        || name.starts_with("xl/tables/")
        || name == "xl/sharedStrings.xml"
        || name == "xl/styles.xml"
}

fn read_zip_records(path: &Path) -> Result<(BTreeMap<String, ZipRecord>, Vec<String>)> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut archive = ZipArchive::new(file)?;
    let mut records = BTreeMap::new();
    let mut order = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .with_context(|| format!("Failed to read ZIP entry {name} in {}", path.display()))?;
        let record = ZipRecord {
            data,
            compression: entry.compression(),
            modified: entry.last_modified().unwrap_or_else(DateTime::default),
            is_dir: entry.is_dir(),
        };
        order.push(name.clone());
        records.insert(name, record);
    }

    Ok((records, order))
}

fn write_zip_records(
    path: &Path,
    records: BTreeMap<String, ZipRecord>,
    order: &[String],
) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp = Builder::new()
        .prefix("tabula_merge_xls")
        .tempfile_in(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let file = temp
        .reopen()
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let mut writer = ZipWriter::new(file);
    for name in order {
        if let Some(record) = records.get(name) {
            let options = FileOptions::<()>::default()
                .compression_method(record.compression)
                .last_modified_time(record.modified);
            if record.is_dir {
                writer.add_directory(name, options)?;
            } else {
                writer.start_file(name, options)?;
                writer.write_all(&record.data)?;
            }
        }
    }
    writer.finish()?;
    temp.persist(path)?;
    Ok(())
}

fn workbook_relationships_without_calc_chain(xml: &str) -> Result<String> {
    let doc = Document::parse(xml).context("Workbook relationships XML is invalid")?;
    let mut rels = Vec::new();
    for node in doc.descendants().filter(|n| {
        n.has_tag_name((
            "http://schemas.openxmlformats.org/package/2006/relationships",
            "Relationship",
        ))
    }) {
        let id = node.attribute("Id").context("Relationship missing Id")?;
        let target = node.attribute("Target").context("Relationship missing Target")?;
        let type_name = node.attribute("Type").context("Relationship missing Type")?;
        if type_name.ends_with("/calcChain") {
            continue;
        }
        let mode = node.attribute("TargetMode");
        rels.push((id, type_name, target, mode));
    }
    let mut output = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    output.push_str(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for (id, type_name, target, mode) in rels {
        output.push_str("<Relationship");
        output.push_str(&format!(r#" Id="{id}" Type="{type_name}" Target="{target}""#));
        if let Some(value) = mode {
            output.push_str(&format!(r#" TargetMode="{value}""#));
        }
        output.push_str("/>");
    }
    output.push_str("</Relationships>");
    Ok(output)
}

fn merged_content_types(template: &[u8], updated: &[u8]) -> Result<Vec<u8>> {
    let mut defaults = BTreeMap::new();
    let mut overrides = BTreeMap::new();
    for data in [template, updated] {
        let text = std::str::from_utf8(data).context("Content types are not valid UTF-8")?;
        let doc = Document::parse(text).context("Failed to parse content types XML")?;
        for node in doc.descendants().filter(|n| {
            n.has_tag_name((
                "http://schemas.openxmlformats.org/package/2006/content-types",
                "Default",
            ))
        }) {
            let ext =
                node.attribute("Extension").context("Content types entry missing Extension")?;
            let content_type =
                node.attribute("ContentType").context("Content types entry missing ContentType")?;
            defaults.entry(ext.to_string()).or_insert_with(|| content_type.to_string());
        }
        for node in doc.descendants().filter(|n| {
            n.has_tag_name((
                "http://schemas.openxmlformats.org/package/2006/content-types",
                "Override",
            ))
        }) {
            let part =
                node.attribute("PartName").context("Content types entry missing PartName")?;
            let content_type =
                node.attribute("ContentType").context("Content types entry missing ContentType")?;
            overrides.entry(part.to_string()).or_insert_with(|| content_type.to_string());
        }
    }
    overrides.remove("/xl/calcChain.xml");

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push_str(r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#);
    for (ext, content_type) in defaults {
        xml.push_str(&format!(r#"<Default Extension="{ext}" ContentType="{content_type}"/>"#));
    }
    for (part, content_type) in overrides {
        xml.push_str(&format!(r#"<Override PartName="{part}" ContentType="{content_type}"/>"#));
    }
    xml.push_str("</Types>");
    Ok(xml.into_bytes())
}

fn ensure_recalc_on_open(path: &Path) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let parent = path.parent().unwrap_or(Path::new("."));
    let patched = Builder::new().prefix("tabula_calc_patch").tempfile_in(parent)?;
    let mut writer = ZipWriter::new(patched);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name == "xl/calcChain.xml" {
            continue;
        }
        let options = FileOptions::<()>::default()
            .compression_method(entry.compression())
            .last_modified_time(entry.last_modified().unwrap_or_else(DateTime::default));
        if entry.is_dir() {
            writer.add_directory(name.clone(), options)?;
            continue;
        }
        writer.start_file(name.clone(), options)?;
        if name == "xl/workbook.xml" {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            let calc_tag =
                r#"<calcPr calcId="122211" calcMode="auto" fullCalcOnLoad="1" forceFullCalc="1"/>"#;
            if let Some(start) = contents.find("<calcPr") {
                if let Some(end) = contents[start..].find("/>") {
                    let end_idx = start + end + 2;
                    contents.replace_range(start..end_idx, calc_tag);
                }
            } else if let Some(pos) = contents.rfind("</workbook>") {
                contents.insert_str(pos, calc_tag);
            }
            writer.write_all(contents.as_bytes())?;
        } else if name == "xl/_rels/workbook.xml.rels" {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            let filtered = workbook_relationships_without_calc_chain(&contents)?;
            writer.write_all(filtered.as_bytes())?;
        } else if name == "[Content_Types].xml" {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).with_context(|| {
                format!("Failed to read ZIP entry {name} in {}", path.display())
            })?;
            let merged = merged_content_types(&contents, &contents)?;
            writer.write_all(&merged)?;
        } else {
            std::io::copy(&mut entry, &mut writer)?;
        }
    }
    let patched_file = writer.finish()?;
    patched_file.persist(path)?;
    Ok(())
}
