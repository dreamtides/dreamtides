use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tempfile::NamedTempFile;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::core::paths;

const MANIFEST_FILENAME: &str = "_xlsm_manifest.json";

pub fn rebuild_images(xlsm_path: Option<PathBuf>) -> Result<()> {
    let source = resolve_xlsm_path(xlsm_path)?;
    let git_root = paths::git_root_for(&source)?;
    let image_cache_dir = paths::image_cache_dir_for(&git_root);
    let manifest_path = manifest_path_for(&image_cache_dir);

    let manifest = read_manifest(&manifest_path)?;
    if manifest.version != 1 {
        bail!("Unsupported manifest version {}", manifest.version);
    }

    let records = read_zip(&source)
        .with_context(|| format!("File {} is not a valid XLSM archive", source.display()))?;
    let (updated_records, file_order, restored) =
        restore_records(records, &manifest, &image_cache_dir)?;
    if restored == 0 {
        bail!("No cached images were restored");
    }

    write_zip(&source, updated_records, &file_order)
}

#[derive(Deserialize)]
struct ImageInfo {
    hash: String,
    size: usize,
}

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    file_order: Vec<String>,
    images: BTreeMap<String, ImageInfo>,
}

struct FileRecord {
    name: String,
    data: Vec<u8>,
    compression: CompressionMethod,
}

fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

fn manifest_path_for(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MANIFEST_FILENAME)
}

fn read_manifest(path: &Path) -> Result<Manifest> {
    let data =
        fs::read(path).with_context(|| format!("Cannot open manifest file {}", path.display()))?;
    serde_json::from_slice(&data).context("Failed to parse manifest file")
}

fn read_zip(path: &Path) -> Result<(Vec<FileRecord>, Vec<String>)> {
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

fn restore_records(
    records: (Vec<FileRecord>, Vec<String>),
    manifest: &Manifest,
    cache_dir: &Path,
) -> Result<(Vec<FileRecord>, Vec<String>, usize)> {
    let (files, current_order) = records;
    let mut updated = Vec::new();
    let mut present = HashSet::new();
    let mut restored = 0usize;

    for mut record in files {
        present.insert(record.name.clone());
        if let Some(info) = manifest.images.get(&record.name) {
            let cache_path = cache_dir.join(&info.hash);
            let data = fs::read(&cache_path)
                .with_context(|| format!("Cannot read cached image {}", cache_path.display()))?;
            if data.len() != info.size {
                bail!(
                    "Cached image {} has unexpected size {} (expected {})",
                    cache_path.display(),
                    data.len(),
                    info.size
                );
            }
            record.data = data;
            restored += 1;
        }
        updated.push(record);
    }

    for name in manifest.images.keys() {
        if !present.contains(name) {
            bail!("Spreadsheet is missing image entry {}", name);
        }
    }

    let mut file_order = manifest.file_order.clone();
    for name in current_order {
        if !file_order.contains(&name) {
            file_order.push(name);
        }
    }

    for name in &file_order {
        if !present.contains(name.as_str()) {
            bail!("Spreadsheet is missing entry {}", name);
        }
    }

    Ok((updated, file_order, restored))
}

fn write_zip(path: &Path, records: Vec<FileRecord>, file_order: &[String]) -> Result<()> {
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
