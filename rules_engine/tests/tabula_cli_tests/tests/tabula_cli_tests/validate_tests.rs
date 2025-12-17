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
        ValidateConfig {
            applescript: false,
            strip_images: false,
            report_all: false,
            verbose: false,
        },
        Some(toml_dir),
        Some(xlsm_path),
    );

    assert!(result.is_ok());
}

#[test]
fn validate_errors_when_applescript_requested() {
    let result = validate::validate(
        ValidateConfig {
            applescript: true,
            strip_images: false,
            report_all: false,
            verbose: false,
        },
        None,
        None,
    );
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(message.contains("AppleScript validation not implemented yet"));
}
