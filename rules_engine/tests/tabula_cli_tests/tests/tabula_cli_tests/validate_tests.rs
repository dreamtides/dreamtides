use std::fs;

use tabula_cli::commands::validate::ValidateConfig;
use tabula_cli::commands::{build_toml, validate};
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;

#[test]
fn validate_round_trip_succeeds() {
    let temp_dir = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir");
    let assets_dir = temp_dir.path().join("client/Assets/StreamingAssets");
    fs::create_dir_all(&assets_dir).expect("assets dir");
    let xlsm_path = assets_dir.join("Tabula.xlsm");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("spreadsheet");

    let toml_dir = assets_dir.join("Tabula");
    build_toml::build_toml(Some(xlsm_path.clone()), Some(toml_dir.clone())).expect("build toml");

    let result = validate::validate(
        ValidateConfig { strip_images: false, report_all: false, verbose: false },
        Some(toml_dir),
        Some(xlsm_path),
    );

    if let Err(err) = result {
        panic!("{err}");
    }
}

#[test]
fn validate_accepts_numeric_strings() {
    let temp_dir = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir");
    let assets_dir = temp_dir.path().join("client/Assets/StreamingAssets");
    fs::create_dir_all(&assets_dir).expect("assets dir");
    let xlsm_path = assets_dir.join("Tabula.xlsm");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("spreadsheet");

    let toml_dir = assets_dir.join("Tabula");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Carol"
count = "5"
active = true

[[test-table]]
name = "Dave"
count = "7"
active = false
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    let result = validate::validate(
        ValidateConfig { strip_images: false, report_all: false, verbose: false },
        Some(toml_dir),
        Some(xlsm_path),
    );

    if let Err(err) = result {
        panic!("{err}");
    }
}

#[test]
fn validate_strip_images_round_trip_succeeds() {
    let temp_dir = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir");
    let assets_dir = temp_dir.path().join("client/Assets/StreamingAssets");
    fs::create_dir_all(&assets_dir).expect("assets dir");
    let xlsm_path = assets_dir.join("Tabula.xlsm");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsm_path).expect("spreadsheet");
    tabula_cli_test_utils::add_media_entries(&xlsm_path, &[
        ("xl/media/image1.jpg", b"img-one"),
        ("xl/media/image2.png", b"img-two"),
    ])
    .expect("media");

    let toml_dir = assets_dir.join("Tabula");
    build_toml::build_toml(Some(xlsm_path.clone()), Some(toml_dir.clone())).expect("build toml");

    let result = validate::validate(
        ValidateConfig { strip_images: true, report_all: false, verbose: false },
        Some(toml_dir),
        Some(xlsm_path),
    );

    if let Err(err) = result {
        panic!("{err}");
    }
}
