use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tempfile::NamedTempFile;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::core::paths;

mod cache;
mod url;

pub fn rebuild_images(xlsm_path: Option<PathBuf>, from_urls: bool, auto: bool) -> Result<()> {
    let source = resolve_xlsm_path(xlsm_path)?;
    if from_urls && auto {
        bail!("--from-urls cannot be combined with --auto");
    }
    if auto {
        match cache::rebuild_from_cache(&source) {
            Ok(()) => Ok(()),
            Err(err) => {
                eprintln!("warning: Cache rebuild failed: {err}; falling back to IMAGE() download");
                url::rebuild_from_urls(&source)
                    .with_context(|| format!("Cache rebuild failed: {err}"))
            }
        }
    } else if from_urls {
        url::rebuild_from_urls(&source)
    } else {
        cache::rebuild_from_cache(&source)
    }
}

#[derive(Clone)]
pub(super) struct FileRecord {
    pub name: String,
    pub data: Vec<u8>,
    pub compression: CompressionMethod,
}

pub(super) fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

pub(super) fn read_zip(path: &Path) -> Result<(Vec<FileRecord>, Vec<String>)> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut archive = ZipArchive::new(file)?;
    let mut file_order = Vec::new();
    let mut dirs = Vec::new();
    let mut records = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        file_order.push(name.clone());
        if entry.is_dir() {
            dirs.push(name);
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let compression = entry.compression();
        records.push(FileRecord { name, data, compression });
    }

    for dir in dirs {
        records.push(FileRecord {
            name: dir,
            data: Vec::new(),
            compression: CompressionMethod::Stored,
        });
    }

    Ok((records, file_order))
}

pub(super) fn write_zip(
    path: &Path,
    records: Vec<FileRecord>,
    file_order: &[String],
) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
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

    for name in file_order {
        if let Some(record) = record_map.get(name) {
            if name.ends_with('/') {
                start_dir(&mut writer, &record.name)?;
                continue;
            }
            start_entry(&mut writer, &record.name, record.compression)?;
            writer.write_all(&record.data)?;
        }
    }

    writer.finish()?;
    temp.persist(path)?;
    Ok(())
}

fn start_entry(
    writer: &mut ZipWriter<std::fs::File>,
    name: &str,
    compression: CompressionMethod,
) -> Result<()> {
    let time =
        zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).context("Invalid ZIP timestamp")?;
    let options =
        FileOptions::<()>::default().compression_method(compression).last_modified_time(time);
    writer.start_file(name, options)?;
    Ok(())
}

fn start_dir(writer: &mut ZipWriter<std::fs::File>, name: &str) -> Result<()> {
    let time =
        zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).context("Invalid ZIP timestamp")?;
    let options = FileOptions::<()>::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(time);
    writer.add_directory(name, options)?;
    Ok(())
}
