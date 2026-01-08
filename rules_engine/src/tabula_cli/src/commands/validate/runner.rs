use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tempfile::{Builder, NamedTempFile};
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use super::{toml_compare, workbook_snapshot, xlsm_toml_compare};
use crate::commands::rebuild_images::rebuild;
use crate::commands::{build_xls, strip_images};
use crate::core::{column_names, excel_reader, paths, toml_data};

#[derive(Clone, Copy)]
pub struct ValidateConfig {
    pub strip_images: bool,
    pub report_all: bool,
    pub verbose: bool,
}

pub fn validate(
    config: ValidateConfig,
    toml_dir: Option<PathBuf>,
    xlsm_path: Option<PathBuf>,
) -> Result<()> {
    let toml_dir = resolve_toml_dir(toml_dir)?;
    let template = resolve_xlsm_path(xlsm_path)?;
    if !template.exists() {
        eprintln!("Skipping validation: XLSM template not found at {}", template.display());
        return Ok(());
    }
    let git_root = paths::git_root_for(&template)?;
    let temp_dir = Builder::new().prefix("tabula_validate").tempdir_in(&git_root)?;
    if config.strip_images {
        validate_with_strip_images(config, &toml_dir, &template, temp_dir.path())
    } else {
        validate_standard(config, &toml_dir, &template, temp_dir.path())
    }
}

fn validate_standard(
    config: ValidateConfig,
    toml_dir: &Path,
    template: &Path,
    temp_root: &Path,
) -> Result<()> {
    let mut errors = Vec::new();
    xlsm_toml_compare::compare_xlsm_to_toml(template, toml_dir, config.report_all, &mut errors)?;
    run_roundtrip(config, toml_dir, template, temp_root, &mut errors)?;
    if !errors.is_empty() {
        bail!("{}", errors.join("\n"));
    }
    Ok(())
}

fn validate_with_strip_images(
    config: ValidateConfig,
    toml_dir: &Path,
    template: &Path,
    temp_root: &Path,
) -> Result<()> {
    let mut errors = Vec::new();
    xlsm_toml_compare::compare_xlsm_to_toml(template, toml_dir, config.report_all, &mut errors)?;
    let stripped_path = temp_root.join(output_file_name(template, "stripped"));
    strip_images::strip_images(Some(template.to_path_buf()), Some(stripped_path.clone()))?;
    let placeholder_media = media_files(&stripped_path)?;
    let roundtrip_path = run_roundtrip(config, toml_dir, &stripped_path, temp_root, &mut errors)?;
    let rebuild_target = temp_root.join(output_file_name(template, "rebuilt"));
    fs::copy(&roundtrip_path, &rebuild_target).with_context(|| {
        format!(
            "Cannot write to output directory {}",
            rebuild_target.parent().unwrap_or(Path::new(".")).display()
        )
    })?;
    ensure_media_entries(&rebuild_target, &placeholder_media)?;
    copy_missing_entries(&stripped_path, &rebuild_target)?;
    rebuild::rebuild_images(Some(rebuild_target.clone()), false, false)?;
    workbook_snapshot::compare_workbooks(template, &rebuild_target, config, &mut errors)?;
    compare_media_files(template, &rebuild_target, config, &mut errors)?;
    if !errors.is_empty() {
        bail!("{}", errors.join("\n"));
    }
    Ok(())
}

fn run_roundtrip(
    config: ValidateConfig,
    toml_dir: &Path,
    template: &Path,
    temp_root: &Path,
    errors: &mut Vec<String>,
) -> Result<PathBuf> {
    let output_path = temp_root.join(output_file_name(template, "roundtrip"));
    build_xls::build_xls(
        false,
        Some(toml_dir.to_path_buf()),
        Some(template.to_path_buf()),
        Some(output_path.clone()),
    )?;
    let roundtrip_toml_dir = temp_root.join("toml");
    extract_tables_to_dir(&output_path, &roundtrip_toml_dir)?;
    toml_compare::compare_toml_dirs(toml_dir, &roundtrip_toml_dir, config.report_all, errors)?;
    workbook_snapshot::compare_workbooks(template, &output_path, config, errors)?;
    Ok(output_path)
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

fn extract_tables_to_dir(xlsm_path: &Path, output_dir: &Path) -> Result<()> {
    let tables = excel_reader::extract_tables(xlsm_path)?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Cannot write to output directory {}", output_dir.display()))?;
    for table in tables {
        let toml_string = toml_data::table_to_toml(&table)?;
        let file_name = format!("{}.toml", column_names::normalize_table_name(table.name.as_str()));
        let file_path = output_dir.join(file_name);
        fs::write(&file_path, toml_string).with_context(|| {
            format!("Cannot write to output directory {}", output_dir.display())
        })?;
    }
    Ok(())
}

fn output_file_name(template: &Path, prefix: &str) -> String {
    let base = template
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output.xlsm".to_string());
    if prefix.is_empty() { base } else { format!("{prefix}_{base}") }
}

fn compare_media_files(
    original: &Path,
    rebuilt: &Path,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    let original_media = media_files(original)?;
    let rebuilt_media = media_files(rebuilt)?;
    for (name, data) in &original_media {
        let Some(actual) = rebuilt_media.get(name) else {
            if record_error(
                errors,
                config.report_all,
                format!("Media file '{name}' missing after rebuild"),
            ) {
                return Ok(());
            }
            continue;
        };
        if data != actual
            && record_error(
                errors,
                config.report_all,
                format!("Media file '{name}' differs after rebuild"),
            )
        {
            return Ok(());
        }
    }
    for name in rebuilt_media.keys() {
        if !original_media.contains_key(name)
            && record_error(
                errors,
                config.report_all,
                format!("Unexpected media file '{name}' after rebuild"),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn ensure_media_entries(target: &Path, expected: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    let (records, mut order) = read_zip_records(target)?;
    let mut record_map: BTreeMap<_, _> =
        records.into_iter().map(|record| (record.name.clone(), record)).collect();
    let mut changed = false;
    for (name, data) in expected {
        if !order.contains(name) {
            order.push(name.clone());
            changed = true;
        }
        let needs_update = match record_map.get(name) {
            Some(record) => record.is_dir || record.data != *data,
            None => true,
        };
        if needs_update {
            record_map.insert(name.clone(), ZipRecord {
                name: name.clone(),
                data: data.clone(),
                compression: CompressionMethod::Stored,
                is_dir: false,
            });
            changed = true;
        }
    }
    if changed {
        write_zip_records(target, record_map.into_values().collect(), &order)
    } else {
        Ok(())
    }
}

fn copy_missing_entries(source: &Path, target: &Path) -> Result<()> {
    let (target_records, mut target_order) = read_zip_records(target)?;
    let target_names: BTreeSet<_> = target_records.iter().map(|r| r.name.clone()).collect();
    let mut target_map: BTreeMap<_, _> =
        target_records.into_iter().map(|r| (r.name.clone(), r)).collect();
    let (source_records, source_order) = read_zip_records(source)?;
    let source_map: BTreeMap<_, _> =
        source_records.into_iter().map(|r| (r.name.clone(), r)).collect();
    let mut changed = false;
    for name in source_order {
        if target_names.contains(&name) {
            continue;
        }
        if let Some(record) = source_map.get(&name) {
            target_order.push(name.clone());
            target_map.insert(name.clone(), record.clone());
            changed = true;
        }
    }
    if changed {
        write_zip_records(target, target_map.into_values().collect(), &target_order)?;
    }
    Ok(())
}

fn media_files(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut map = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if !name.starts_with("xl/media/") {
            continue;
        }
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .with_context(|| format!("Failed to read ZIP entry {name} in {}", path.display()))?;
        map.insert(name, data);
    }
    Ok(map)
}

#[derive(Clone)]
struct ZipRecord {
    name: String,
    data: Vec<u8>,
    compression: CompressionMethod,
    is_dir: bool,
}

fn read_zip_records(path: &Path) -> Result<(Vec<ZipRecord>, Vec<String>)> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut records = Vec::new();
    let mut order = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        order.push(name.clone());
        if entry.is_dir() {
            records.push(ZipRecord {
                name,
                data: Vec::new(),
                compression: CompressionMethod::Stored,
                is_dir: true,
            });
            continue;
        }
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .with_context(|| format!("Failed to read ZIP entry {name} in {}", path.display()))?;
        records.push(ZipRecord { name, data, compression: entry.compression(), is_dir: false });
    }
    Ok((records, order))
}

fn write_zip_records(path: &Path, records: Vec<ZipRecord>, order: &[String]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let file = temp
        .reopen()
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let mut writer = ZipWriter::new(file);
    let mut record_map = BTreeMap::new();
    for record in records {
        record_map.insert(record.name.clone(), record);
    }
    let time =
        DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).context("Invalid ZIP timestamp")?;
    for name in order {
        if let Some(record) = record_map.get(name) {
            let options = FileOptions::<()>::default()
                .compression_method(record.compression)
                .last_modified_time(time);
            if record.is_dir || name.ends_with('/') {
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

pub(super) fn record_error(errors: &mut Vec<String>, report_all: bool, message: String) -> bool {
    errors.push(message);
    !report_all
}
