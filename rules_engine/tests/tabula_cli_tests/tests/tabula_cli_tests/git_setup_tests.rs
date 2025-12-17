use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use tabula_cli::commands::git_setup::{self, Hook};
use tabula_cli::commands::build_toml;
use tabula_cli::commands::strip_images;
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use zip::ZipArchive;

#[test]
fn git_setup_installs_hooks_and_gitattributes() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);

    git_setup::git_setup_for_root(root).expect("setup");

    let gitattributes =
        fs::read_to_string(root.join(".gitattributes")).expect("gitattributes contents");
    assert!(gitattributes
        .contains("client/Assets/StreamingAssets/TabulaData.xlsm filter=lfs diff=lfs merge=lfs -text"));

    let pre_commit = fs::read_to_string(root.join(".git/hooks/pre-commit")).expect("pre-commit");
    let post_checkout =
        fs::read_to_string(root.join(".git/hooks/post-checkout")).expect("post-checkout");
    let post_merge = fs::read_to_string(root.join(".git/hooks/post-merge")).expect("post-merge");

    assert!(pre_commit.contains("git-hook pre-commit"));
    assert!(post_checkout.contains("git-hook post-checkout"));
    assert!(post_merge.contains("git-hook post-merge"));
    assert!(root.join(".git/xlsm_image_cache").is_dir());
}

#[test]
fn git_setup_errors_when_core_hooks_path_set_elsewhere() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);
    Command::new("git")
        .current_dir(root)
        .arg("config")
        .arg("core.hooksPath")
        .arg(".githooks")
        .status()
        .expect("set hooksPath");

    let result = git_setup::git_setup_for_root(root);
    let message = result.unwrap_err().to_string();
    assert!(message.contains("core.hooksPath"));
}

#[test]
fn pre_commit_builds_toml_strips_and_stages_changes() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);
    git_setup::git_setup_for_root(root).expect("setup");

    let xlsm_path = root.join("client/Assets/StreamingAssets/Tabula.xlsm");
    fs::create_dir_all(xlsm_path.parent().unwrap()).expect("xlsm dir");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("sheet");
    tabula_cli_test_utils::add_media_entries(
        &xlsm_path,
        &[("xl/media/image1.jpg", b"img-one"), ("xl/media/image2.png", b"img-two")],
    )
    .expect("media");

    git_setup::run_hook_for_root(Hook::PreCommit, root).expect("pre-commit");

    let data_path = root.join("client/Assets/StreamingAssets/TabulaData.xlsm");
    let mut data_archive =
        ZipArchive::new(fs::File::open(&data_path).expect("open zip")).expect("zip archive");
    let mut data = Vec::new();
    data_archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut data)
        .expect("read image1");
    assert_eq!(data, strip_images::PLACEHOLDER_JPEG);

    let mut xlsm_archive =
        ZipArchive::new(fs::File::open(&xlsm_path).expect("open zip")).expect("zip archive");
    let mut original = Vec::new();
    xlsm_archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut original)
        .expect("read image1");
    assert_ne!(original, strip_images::PLACEHOLDER_JPEG);

    let staged = Command::new("git")
        .current_dir(root)
        .arg("show")
        .arg(":client/Assets/StreamingAssets/TabulaData.xlsm")
        .output()
        .expect("git show");
    assert!(staged.status.success());
    let staged_text = String::from_utf8_lossy(&staged.stdout);
    assert!(staged_text.contains("git-lfs.github.com/spec/v1"));

    let toml_path = root.join("client/Assets/StreamingAssets/Tabula/test-table.toml");
    let toml_content = fs::read_to_string(&toml_path).expect("toml file");
    assert!(toml_content.contains("Alice"));
    assert!(toml_content.contains("Bob"));

    let status = Command::new("git")
        .current_dir(root)
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("git status");
    let output = String::from_utf8_lossy(&status.stdout);
    assert!(!output.contains("Tabula.xlsm"));
    assert!(output.contains("client/Assets/StreamingAssets/TabulaData.xlsm"));
    assert!(output.contains("client/Assets/StreamingAssets/Tabula/test-table.toml"));
}

#[test]
fn pre_commit_fails_when_toml_is_newer_than_spreadsheet() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);

    let xlsm_path = root.join("client/Assets/StreamingAssets/Tabula.xlsm");
    fs::create_dir_all(xlsm_path.parent().unwrap()).expect("xlsm dir");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("sheet");
    tabula_cli_test_utils::add_media_entries(
        &xlsm_path,
        &[("xl/media/image1.jpg", b"img-one")],
    )
    .expect("media");

    thread::sleep(Duration::from_secs(1));
    let toml_dir = root.join("client/Assets/StreamingAssets/Tabula");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    fs::write(toml_dir.join("test-table.toml"), "[]").expect("write toml");

    let result = git_setup::run_hook_for_root(Hook::PreCommit, root);
    let message = result.unwrap_err().to_string();
    assert!(message.contains("newer than"));
    assert!(message.contains("build-xls"));

    let mut archive = ZipArchive::new(fs::File::open(&xlsm_path).expect("open zip"))
        .expect("zip archive");
    let mut data = Vec::new();
    archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut data)
        .expect("read image1");
    assert_ne!(data, strip_images::PLACEHOLDER_JPEG);
}

#[test]
fn pre_commit_allows_newer_matching_toml() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);
    git_setup::git_setup_for_root(root).expect("setup");

    let xlsm_path = root.join("client/Assets/StreamingAssets/Tabula.xlsm");
    fs::create_dir_all(xlsm_path.parent().unwrap()).expect("xlsm dir");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("sheet");
    tabula_cli_test_utils::add_media_entries(
        &xlsm_path,
        &[("xl/media/image1.jpg", b"img-one")],
    )
    .expect("media");

    build_toml::build_toml(Some(xlsm_path.clone()), Some(root.join("client/Assets/StreamingAssets/Tabula"))).expect("build-toml");

    git_setup::run_hook_for_root(Hook::PreCommit, root).expect("pre-commit");
}

#[test]
fn rebuild_hook_restores_images_from_cache() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    init_repo(root);
    git_setup::git_setup_for_root(root).expect("setup");

    let xlsm_path = root.join("client/Assets/StreamingAssets/Tabula.xlsm");
    let data_path = root.join("client/Assets/StreamingAssets/TabulaData.xlsm");
    fs::create_dir_all(xlsm_path.parent().unwrap()).expect("xlsm dir");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("sheet");
    let original_one = b"img-one".to_vec();
    let original_two = b"img-two".to_vec();
    tabula_cli_test_utils::add_media_entries(
        &xlsm_path,
        &[("xl/media/image1.jpg", &original_one), ("xl/media/image2.png", &original_two)],
    )
    .expect("media");

    strip_images::strip_images(Some(xlsm_path.clone()), Some(data_path.clone())).expect("strip");
    fs::remove_file(&xlsm_path).expect("remove original");
    git_setup::run_hook_for_root(Hook::PostCheckout, root).expect("rebuild");

    let mut archive = ZipArchive::new(fs::File::open(&xlsm_path).expect("open zip"))
        .expect("zip archive");
    let mut image_one = Vec::new();
    let mut image_two = Vec::new();
    archive
        .by_name("xl/media/image1.jpg")
        .expect("image1")
        .read_to_end(&mut image_one)
        .expect("read image1");
    archive
        .by_name("xl/media/image2.png")
        .expect("image2")
        .read_to_end(&mut image_two)
        .expect("read image2");
    assert_eq!(image_one, original_one);
    assert_eq!(image_two, original_two);
}

fn init_repo(root: &Path) {
    let status = Command::new("git").arg("init").arg(root).status().expect("git init");
    assert!(status.success());
}
