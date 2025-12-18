use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::core::paths;

pub const PLACEHOLDER_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
    0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
    0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
    0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
    0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
    0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D, 0x01, 0x02, 0x03, 0x00,
    0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32,
    0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72,
    0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35,
    0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93, 0x94,
    0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2,
    0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9,
    0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6,
    0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA,
    0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0xFB, 0xD5, 0xDB, 0x20, 0xA8, 0xA2, 0x80, 0x3F,
    0xFF, 0xD9,
];

const MANIFEST_FILENAME: &str = "_xlsm_manifest.json";

#[derive(Serialize)]
struct ImageInfo {
    hash: String,
    size: usize,
    original_name: String,
}

#[derive(Serialize)]
struct Manifest {
    version: u32,
    file_order: Vec<String>,
    images: BTreeMap<String, ImageInfo>,
    source_file: String,
}

struct FileRecord {
    name: String,
    data: Vec<u8>,
    compression: CompressionMethod,
}

pub fn strip_images(xlsm_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<()> {
    let source = resolve_xlsm_path(xlsm_path)?;
    let output = resolve_output_path(&source, output_path)?;
    let git_root = paths::git_root_for(&source)?;
    let image_cache_dir = paths::image_cache_dir_for(&git_root);
    let manifest_path = manifest_path_for(&image_cache_dir);

    let records = read_zip(&source)
        .with_context(|| format!("File {} is not a valid XLSM archive", source.display()))?;

    let (updated_records, manifest, image_count) =
        process_records(records, &image_cache_dir, &source)?;

    if image_count == 0 {
        bail!("No embedded images found");
    }

    write_zip(&output, updated_records, &manifest.file_order)?;
    write_manifest_file(&manifest, &manifest_path)?;

    Ok(())
}

fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

fn resolve_output_path(source: &Path, output_path: Option<PathBuf>) -> Result<PathBuf> {
    match output_path {
        Some(path) => Ok(path),
        None => Ok(source.to_path_buf()),
    }
}

fn manifest_path_for(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MANIFEST_FILENAME)
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
        entry
            .read_to_end(&mut data)
            .with_context(|| format!("Failed to read ZIP entry {name} in {}", path.display()))?;
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

fn process_records(
    records: (Vec<FileRecord>, Vec<String>),
    image_cache_dir: &Path,
    source: &Path,
) -> Result<(Vec<FileRecord>, Manifest, usize)> {
    let (files, file_order) = records;
    let mut output_records = Vec::new();
    let mut images = BTreeMap::new();
    let mut image_count = 0usize;

    for record in files {
        if is_image(&record.name) {
            image_count += 1;
            let hash = hash_bytes(&record.data);
            let cache_path = image_cache_dir.join(&hash);
            if !cache_path.exists() {
                fs::create_dir_all(image_cache_dir).with_context(|| {
                    format!("Cannot write to output directory {}", image_cache_dir.display())
                })?;
                fs::write(&cache_path, &record.data).with_context(|| {
                    format!("Cannot write to output directory {}", image_cache_dir.display())
                })?;
            }

            let info = ImageInfo {
                hash,
                size: record.data.len(),
                original_name: Path::new(&record.name)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| record.name.clone()),
            };
            images.insert(record.name.clone(), info);

            output_records.push(FileRecord {
                name: record.name,
                data: PLACEHOLDER_JPEG.to_vec(),
                compression: CompressionMethod::Stored,
            });
        } else {
            output_records.push(record);
        }
    }

    let manifest = Manifest {
        version: 1,
        file_order,
        images,
        source_file: source
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| source.display().to_string()),
    };

    Ok((output_records, manifest, image_count))
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

fn write_manifest_file(manifest: &Manifest, manifest_path: &Path) -> Result<()> {
    let parent = manifest_path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let data = serde_json::to_vec_pretty(manifest).context("Failed to serialize manifest")?;
    let temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    temp.as_file().write_all(&data)?;
    temp.persist(manifest_path)?;
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

fn is_image(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("xl/media/")
        && (lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".png")
            || lower.ends_with(".gif")
            || lower.ends_with(".emf")
            || lower.ends_with(".wmf"))
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}
