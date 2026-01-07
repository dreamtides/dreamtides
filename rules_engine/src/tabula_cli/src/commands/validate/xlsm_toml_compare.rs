use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use toml::{Table, Value};

use super::runner;
use crate::core::{column_names, excel_reader, toml_data};

#[derive(Clone)]
struct ParsedTomlTable {
    normalized_name: String,
    source_name: String,
    value: Value,
}

pub(super) fn compare_xlsm_to_toml(
    xlsm_path: &Path,
    toml_dir: &Path,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    let toml_tables = load_toml_tables(toml_dir)?;
    let xlsm_tables = extract_xlsm_tables(xlsm_path)?;
    let initial_error_count = errors.len();
    for (name, toml_table) in &toml_tables {
        let Some(xlsm_table) = xlsm_tables.get(name) else {
            if runner::record_error(
                errors,
                report_all,
                format!(
                    "XLSM validation failed: table '{}' exists in TOML but not in XLSM",
                    toml_table.source_name
                ),
            ) {
                return Ok(());
            }
            continue;
        };
        compare_tables(toml_table, xlsm_table, report_all, errors)?;
    }
    if let Some(extra) =
        xlsm_tables.values().find(|t| !toml_tables.contains_key(&t.normalized_name))
    {
        runner::record_error(
            errors,
            report_all,
            format!(
                "XLSM validation failed: table '{}' exists in XLSM but not in TOML",
                extra.source_name
            ),
        );
    }
    if errors.len() > initial_error_count {
        errors.push(String::new());
        errors.push(
            "Hint: Run 'just tabula build-xls --output-path client/Assets/StreamingAssets/Tabula.xlsm' to update the XLSM from TOML"
                .to_string(),
        );
    }
    Ok(())
}

fn load_toml_tables(dir: &Path) -> Result<BTreeMap<String, ParsedTomlTable>> {
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
                bail!("Duplicate TOML table '{normalized_name}'");
            }
            tables.insert(normalized_name.clone(), ParsedTomlTable {
                normalized_name,
                source_name: key,
                value: data.clone(),
            });
        }
    }
    Ok(tables)
}

fn extract_xlsm_tables(xlsm_path: &Path) -> Result<BTreeMap<String, ParsedTomlTable>> {
    let table_infos = excel_reader::extract_tables(xlsm_path)?;
    let mut tables = BTreeMap::new();
    for table_info in table_infos {
        let toml_string = toml_data::table_to_toml(&table_info)?;
        let value: Value = toml::from_str(&toml_string).with_context(|| {
            format!("Failed to parse generated TOML for table '{}'", table_info.name)
        })?;
        let value = toml_data::canonicalize_numbers(value);
        let table = value.as_table().cloned().unwrap_or_default();
        for (key, data) in table {
            let normalized_name = column_names::normalize_table_name(key.as_str());
            if tables.contains_key(&normalized_name) {
                bail!("Duplicate XLSM table '{normalized_name}'");
            }
            tables.insert(normalized_name.clone(), ParsedTomlTable {
                normalized_name,
                source_name: key,
                value: data.clone(),
            });
        }
    }
    Ok(tables)
}

fn compare_tables(
    toml_table: &ParsedTomlTable,
    xlsm_table: &ParsedTomlTable,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    match (&toml_table.value, &xlsm_table.value) {
        (Value::Array(toml_rows), Value::Array(xlsm_rows)) => {
            if toml_rows.len() != xlsm_rows.len()
                && runner::record_error(
                    errors,
                    report_all,
                    format!(
                        "XLSM validation failed: table '{}' has {} rows in TOML but {} rows in XLSM",
                        toml_table.source_name,
                        toml_rows.len(),
                        xlsm_rows.len()
                    ),
                )
            {
                return Ok(());
            }
            for (idx, (toml_row, xlsm_row)) in toml_rows.iter().zip(xlsm_rows.iter()).enumerate() {
                match (toml_row, xlsm_row) {
                    (Value::Table(toml_table_row), Value::Table(xlsm_table_row)) => {
                        compare_table_row(
                            toml_table.source_name.as_str(),
                            idx + 1,
                            toml_table_row,
                            xlsm_table_row,
                            report_all,
                            errors,
                        )?;
                    }
                    _ => {
                        if toml_row != xlsm_row
                            && runner::record_error(
                                errors,
                                report_all,
                                format!(
                                    "XLSM validation failed: table '{}' row {} differs: TOML has {} but XLSM has {}",
                                    toml_table.source_name,
                                    idx + 1,
                                    format_value(toml_row),
                                    format_value(xlsm_row)
                                ),
                            )
                        {
                            return Ok(());
                        }
                    }
                }
            }
            Ok(())
        }
        _ => {
            if toml_table.value == xlsm_table.value {
                Ok(())
            } else {
                runner::record_error(
                    errors,
                    report_all,
                    format!(
                        "XLSM validation failed: table '{}' structure differs between TOML and XLSM",
                        toml_table.source_name
                    ),
                );
                Ok(())
            }
        }
    }
}

fn compare_table_row(
    table_name: &str,
    row_idx: usize,
    toml_row: &Table,
    xlsm_row: &Table,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    let toml_keys: BTreeSet<_> =
        toml_row.keys().filter(|k| k.as_str() != "preview").cloned().collect();
    let xlsm_keys: BTreeSet<_> =
        xlsm_row.keys().filter(|k| k.as_str() != "preview").cloned().collect();
    for key in &toml_keys {
        if !xlsm_keys.contains(key)
            && runner::record_error(
                errors,
                report_all,
                format!(
                    "XLSM validation failed: table '{table_name}' row {row_idx} has column '{key}' in TOML but not in XLSM"
                ),
            )
        {
            return Ok(());
        }
    }
    for key in &xlsm_keys {
        if !toml_keys.contains(key)
            && runner::record_error(
                errors,
                report_all,
                format!(
                    "XLSM validation failed: table '{table_name}' row {row_idx} has column '{key}' in XLSM but not in TOML"
                ),
            )
        {
            return Ok(());
        }
    }
    for key in &toml_keys {
        let toml_value = toml_row.get(key).unwrap();
        let xlsm_value = xlsm_row.get(key).unwrap();
        if toml_value != xlsm_value
            && runner::record_error(
                errors,
                report_all,
                format!(
                    "XLSM validation failed: table '{table_name}' row {row_idx} column '{key}': TOML has {} but XLSM has {}",
                    format_value(toml_value),
                    format_value(xlsm_value)
                ),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn format_value(value: &Value) -> String {
    value.to_string()
}
