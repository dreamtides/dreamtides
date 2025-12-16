use std::fs;
use std::fs::File;
use std::io::Read;

use sha2::{Digest, Sha256};
use tabula_cli::commands::strip_images::{self, PLACEHOLDER_JPEG};
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use zip::ZipArchive;

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn strip_images_replaces_and_caches() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsm_path = temp_dir.path().join("input.xlsm");
    let (order, img1, img2) =
        tabula_cli_test_utils::create_xlsm_with_images(&xlsm_path).expect("xlsm");

    let output = temp_dir.path().join("out.xlsm");
    strip_images::strip_images(Some(xlsm_path.clone()), Some(output.clone())).expect("strip");

    let mut archive = ZipArchive::new(File::open(&output).expect("open output")).expect("zip");
    let mut names = Vec::new();
    for i in 0..archive.len() {
        names.push(archive.by_index(i).expect("entry").name().to_string());
    }
    assert_eq!(names, order);

    let mut img1_data = Vec::new();
    archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut img1_data)
        .expect("read");
    assert_eq!(img1_data, PLACEHOLDER_JPEG);
    let mut img2_data = Vec::new();
    archive
        .by_name("xl/media/image2.png")
        .expect("image2")
        .read_to_end(&mut img2_data)
        .expect("read");
    assert_eq!(img2_data, PLACEHOLDER_JPEG);

    let manifest_path = temp_dir.path().join(".git/xlsm_image_cache/_xlsm_manifest.json");
    assert!(manifest_path.exists());
    let manifest_data = fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_data).expect("json");
    let images = manifest.get("images").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let info1 = images.get("xl/media/image1.jpg").and_then(|v| v.as_object()).expect("info1");
    let info2 = images.get("xl/media/image2.png").and_then(|v| v.as_object()).expect("info2");
    assert_eq!(info1.get("size").and_then(|v| v.as_u64()), Some(img1.len() as u64));
    assert_eq!(info2.get("size").and_then(|v| v.as_u64()), Some(img2.len() as u64));
    let hash1 = hash_bytes(&img1);
    let hash2 = hash_bytes(&img2);
    assert_eq!(info1.get("hash").and_then(|v| v.as_str()), Some(hash1.as_str()));
    assert_eq!(info2.get("hash").and_then(|v| v.as_str()), Some(hash2.as_str()));

    let cache_dir = git_dir.join("xlsm_image_cache");
    assert!(cache_dir.join(hash1).exists());
    assert!(cache_dir.join(hash2).exists());
}
