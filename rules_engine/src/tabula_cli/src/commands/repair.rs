use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use zip::read::ZipFile;
use zip::write::FileOptions;
use zip::{DateTime, ZipArchive, ZipWriter};

use crate::commands::rebuild_images::rebuild;
use crate::core::paths;

pub fn repair(xlsm_path: Option<PathBuf>, rebuild_media: bool) -> Result<()> {
    let path = resolve_xlsm_path(xlsm_path)?;
    repair_archive(&path)?;
    if rebuild_media { rebuild::rebuild_images(Some(path), false, true) } else { Ok(()) }
}

fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

fn repair_archive(path: &Path) -> Result<()> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("File {} is not a valid XLSM archive", path.display()))?;
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let writer_file = temp
        .reopen()
        .with_context(|| format!("Cannot write to output directory {}", parent.display()))?;
    let mut writer = ZipWriter::new(writer_file);
    let mut fixed = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("File {} is not a valid XLSM archive", path.display()))?;
        let name = entry.name().to_string();
        let options = FileOptions::<()>::default()
            .compression_method(entry.compression())
            .last_modified_time(entry.last_modified().unwrap_or_else(DateTime::default));
        if entry.is_dir() || name.ends_with('/') {
            writer.add_directory(name, options)?;
            continue;
        }
        let (data, corrected) = read_entry(&mut entry, path)?;
        if corrected {
            fixed += 1;
        }
        writer.start_file(name, options)?;
        writer.write_all(&data)?;
    }

    writer.finish()?;
    temp.persist(path)?;
    if fixed > 0 {
        println!("Corrected CRC for {fixed} entries");
    }
    Ok(())
}

fn read_entry(entry: &mut ZipFile<'_, File>, source: &Path) -> Result<(Vec<u8>, bool)> {
    let mut data = Vec::new();
    match entry.read_to_end(&mut data) {
        Ok(_) => Ok((data, false)),
        Err(err) => match err.kind() {
            ErrorKind::InvalidData | ErrorKind::UnexpectedEof if !data.is_empty() => {
                Ok((data, true))
            }
            _ => Err(err).with_context(|| {
                format!("Failed to read ZIP entry {} in {}", entry.name(), source.display())
            }),
        },
    }
}
