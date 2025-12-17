use std::fs;
use std::fs::File;
use std::io::Read;

use tabula_cli::commands::{rebuild_images, strip_images};
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use zip::ZipArchive;

#[test]
fn rebuild_images_restores_from_cache() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsm_path = temp_dir.path().join("input.xlsm");
    let (order, img1, img2) =
        tabula_cli_test_utils::create_xlsm_with_images(&xlsm_path).expect("xlsm");

    let stripped = temp_dir.path().join("stripped.xlsm");
    strip_images::strip_images(Some(xlsm_path.clone()), Some(stripped.clone())).expect("strip");

    rebuild_images::rebuild_images(Some(stripped.clone()), false, false).expect("rebuild");

    let mut archive = ZipArchive::new(File::open(&stripped).expect("open output")).expect("zip");
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
    assert_eq!(img1_data, img1);
    let mut img2_data = Vec::new();
    archive
        .by_name("xl/media/image2.png")
        .expect("image2")
        .read_to_end(&mut img2_data)
        .expect("read");
    assert_eq!(img2_data, img2);
}
