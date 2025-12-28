use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use super::rebuild::{FileRecord, read_zip, write_zip};
use crate::core::paths;

const MANIFEST_FILENAME: &str = "_xlsm_manifest.json";

pub fn rebuild_from_cache(source: &Path) -> Result<()> {
    let git_root = paths::git_root_for(source)?;
    let image_cache_dir = paths::image_cache_dir_for(&git_root);
    let manifest_path = manifest_path_for(&image_cache_dir);

    let manifest = read_manifest(&manifest_path)?;
    if manifest.version != 1 {
        bail!("Unsupported manifest version {}", manifest.version);
    }

    let records = read_zip(source).with_context(|| {
        format!("File {path} is not a valid XLSM archive", path = source.display())
    })?;
    let (updated_records, file_order, restored) =
        restore_records(records, &manifest, &image_cache_dir)?;
    if restored == 0 {
        bail!("No cached images were restored");
    }

    write_zip(source, updated_records, &file_order)
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

fn manifest_path_for(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MANIFEST_FILENAME)
}

fn read_manifest(path: &Path) -> Result<Manifest> {
    let data =
        fs::read(path).with_context(|| format!("Cannot open manifest file {}", path.display()))?;
    serde_json::from_slice(&data).context("Failed to parse manifest file")
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
        if !present.contains(name.as_str()) && !is_optional_entry(name) {
            bail!("Spreadsheet is missing entry {}", name);
        }
    }

    Ok((updated, file_order, restored))
}

fn is_optional_entry(name: &str) -> bool {
    name == "xl/calcChain.xml"
}
