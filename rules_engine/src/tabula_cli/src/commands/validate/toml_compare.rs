use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use toml::{Table, Value};

use super::record_error;
use crate::core::{column_names, toml_data};

#[derive(Clone)]
struct ParsedTomlTable {
    normalized_name: String,
    source_name: String,
    value: Value,
}

pub(super) fn compare_toml_dirs(
    original: &Path,
    roundtrip: &Path,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    let expected = load_toml_tables(original)?;
    let actual = load_toml_tables(roundtrip)?;
    for (name, table) in &expected {
        let Some(actual_table) = actual.get(name) else {
            if record_error(
                errors,
                report_all,
                format!(
                    "Round-trip failed: TOML differs at table '{}': missing table",
                    table.source_name
                ),
            ) {
                return Ok(());
            }
            continue;
        };
        compare_toml_values(table, actual_table, report_all, errors)?;
    }
    if let Some(extra) = actual.values().find(|t| !expected.contains_key(&t.normalized_name)) {
        record_error(
            errors,
            report_all,
            format!(
                "Round-trip failed: TOML differs at table '{}': unexpected table",
                extra.source_name
            ),
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
                bail!("Unexpected TOML table '{normalized_name}' (not present in template)");
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

fn compare_toml_values(
    expected: &ParsedTomlTable,
    actual: &ParsedTomlTable,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    match (&expected.value, &actual.value) {
        (Value::Array(expected_rows), Value::Array(actual_rows)) => {
            if expected_rows.len() != actual_rows.len()
                && record_error(
                    errors,
                    report_all,
                    format!(
                        "Round-trip failed: TOML differs at table '{}', row count {} vs {}",
                        expected.source_name,
                        expected_rows.len(),
                        actual_rows.len()
                    ),
                )
            {
                return Ok(());
            }
            for (idx, (expected_row, actual_row)) in
                expected_rows.iter().zip(actual_rows.iter()).enumerate()
            {
                match (expected_row, actual_row) {
                    (Value::Table(expected_table), Value::Table(actual_table)) => {
                        compare_table_row(
                            expected.source_name.as_str(),
                            idx + 1,
                            expected_table,
                            actual_table,
                            report_all,
                            errors,
                        )?;
                    }
                    _ => {
                        if expected_row != actual_row
                            && record_error(
                                errors,
                                report_all,
                                format!(
                                    "Round-trip failed: TOML differs at table '{}', row {}, expected {} but found {}",
                                    expected.source_name,
                                    idx + 1,
                                    format_value(expected_row),
                                    format_value(actual_row)
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
            if expected.value == actual.value {
                Ok(())
            } else {
                record_error(
                    errors,
                    report_all,
                    format!("Round-trip failed: TOML differs at table '{}'", expected.source_name),
                );
                Ok(())
            }
        }
    }
}

fn compare_table_row(
    table_name: &str,
    row_idx: usize,
    expected: &Table,
    actual: &Table,
    report_all: bool,
    errors: &mut Vec<String>,
) -> Result<()> {
    let expected_keys: BTreeSet<_> = expected.keys().cloned().collect();
    let actual_keys: BTreeSet<_> = actual.keys().cloned().collect();
    for key in &expected_keys {
        if !actual_keys.contains(key)
            && record_error(
                errors,
                report_all,
                format!(
                    "Round-trip failed: TOML differs at table '{table_name}', row {row_idx}, missing column '{key}'"
                ),
            )
        {
            return Ok(());
        }
    }
    for key in &actual_keys {
        if !expected_keys.contains(key)
            && record_error(
                errors,
                report_all,
                format!(
                    "Round-trip failed: TOML differs at table '{table_name}', row {row_idx}, unexpected column '{key}'"
                ),
            )
        {
            return Ok(());
        }
    }
    for key in &expected_keys {
        let expected_value = expected.get(key).unwrap();
        let actual_value = actual.get(key).unwrap();
        if expected_value != actual_value
            && record_error(
                errors,
                report_all,
                format!(
                    "Round-trip failed: TOML differs at table '{table_name}', row {row_idx}, column '{key}': expected {} but found {}",
                    format_value(expected_value),
                    format_value(actual_value)
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
