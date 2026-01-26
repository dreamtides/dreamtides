use std::path::PathBuf;

use tv_lib::cli::{exit_codes, AppPaths, Args};

fn args_with_path(path: PathBuf) -> Args {
    Args { path: Some(path) }
}

fn args_none() -> Args {
    Args { path: None }
}

#[test]
fn test_exit_codes_values() {
    assert_eq!(exit_codes::SUCCESS, 0);
    assert_eq!(exit_codes::INVALID_ARGUMENTS, 1);
    assert_eq!(exit_codes::FILE_ERROR, 2);
}

#[test]
fn test_resolve_single_toml_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.toml");
    std::fs::write(&file_path, "[[cards]]\nid = \"card-1\"\n").unwrap();

    let args = args_with_path(file_path.clone());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0], file_path);
}

#[test]
fn test_resolve_directory_with_multiple_toml_files() {
    let dir = tempfile::tempdir().unwrap();
    let file_a = dir.path().join("alpha.toml");
    let file_b = dir.path().join("beta.toml");
    let file_c = dir.path().join("gamma.toml");
    std::fs::write(&file_a, "[[items]]\nid = \"1\"\n").unwrap();
    std::fs::write(&file_b, "[[items]]\nid = \"2\"\n").unwrap();
    std::fs::write(&file_c, "[[items]]\nid = \"3\"\n").unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files.len(), 3);
    assert_eq!(result.files[0], file_a);
    assert_eq!(result.files[1], file_b);
    assert_eq!(result.files[2], file_c);
}

#[test]
fn test_resolve_directory_sorted_alphabetically() {
    let dir = tempfile::tempdir().unwrap();
    let file_z = dir.path().join("zebra.toml");
    let file_a = dir.path().join("aardvark.toml");
    let file_m = dir.path().join("moose.toml");
    std::fs::write(&file_z, "[[x]]\n").unwrap();
    std::fs::write(&file_a, "[[x]]\n").unwrap();
    std::fs::write(&file_m, "[[x]]\n").unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files[0], file_a);
    assert_eq!(result.files[1], file_m);
    assert_eq!(result.files[2], file_z);
}

#[test]
fn test_resolve_directory_ignores_non_toml_files() {
    let dir = tempfile::tempdir().unwrap();
    let toml_file = dir.path().join("data.toml");
    let txt_file = dir.path().join("readme.txt");
    let json_file = dir.path().join("config.json");
    std::fs::write(&toml_file, "[[items]]\nid = \"1\"\n").unwrap();
    std::fs::write(&txt_file, "hello").unwrap();
    std::fs::write(&json_file, "{}").unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0], toml_file);
}

#[test]
fn test_resolve_nonexistent_file_returns_error() {
    let args = args_with_path(PathBuf::from("/tmp/nonexistent_tv_test_file.toml"));
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"), "Error should mention 'not found': {err}");
}

#[test]
fn test_resolve_nonexistent_directory_returns_error() {
    let args = args_with_path(PathBuf::from("/tmp/nonexistent_tv_test_dir"));
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"), "Error should mention 'not found': {err}");
}

#[test]
fn test_resolve_empty_directory_returns_error() {
    let dir = tempfile::tempdir().unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("No TOML files found"),
        "Error should mention 'No TOML files found': {err}"
    );
}

#[test]
fn test_resolve_directory_with_only_non_toml_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();
    std::fs::write(dir.path().join("data.json"), "{}").unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("No TOML files found"),
        "Error should mention 'No TOML files found': {err}"
    );
}

#[test]
fn test_resolve_none_path_uses_default() {
    let args = args_none();
    let result = tv_lib::cli::resolve_paths(&args);
    // The default path resolution depends on the executable location.
    // In the test environment, the default tabula directory may or may not exist.
    // We just verify the function does not panic and returns some result.
    match result {
        Ok(paths) => {
            assert!(!paths.files.is_empty(), "Default path should find at least one TOML file");
            for file in &paths.files {
                assert!(
                    file.extension().is_some_and(|e| e == "toml"),
                    "All files should have .toml extension: {file:?}"
                );
            }
        }
        Err(err) => {
            assert!(
                err.contains("not found") || err.contains("Could not find"),
                "Default path error should be about missing directory: {err}"
            );
        }
    }
}

#[test]
fn test_resolve_file_path_with_toml_extension() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("cards.toml");
    std::fs::write(&file_path, "[[cards]]\nname = \"Test\"\n").unwrap();

    let args = args_with_path(file_path.clone());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files, vec![file_path]);
}

#[test]
fn test_resolve_directory_excludes_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("data.toml");
    std::fs::write(&file, "[[items]]\n").unwrap();
    std::fs::create_dir(dir.path().join("subdir")).unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0], file);
}

#[test]
fn test_app_paths_files_field() {
    let paths = AppPaths { files: vec![PathBuf::from("/a.toml"), PathBuf::from("/b.toml")] };
    assert_eq!(paths.files.len(), 2);
    assert_eq!(paths.files[0], PathBuf::from("/a.toml"));
    assert_eq!(paths.files[1], PathBuf::from("/b.toml"));
}

#[test]
fn test_app_paths_empty() {
    let paths = AppPaths { files: vec![] };
    assert!(paths.files.is_empty());
}

#[test]
fn test_args_debug_format() {
    let args = Args { path: Some(PathBuf::from("/test.toml")) };
    let debug = format!("{args:?}");
    assert!(debug.contains("test.toml"), "Debug output should contain path: {debug}");
}

#[test]
fn test_args_path_none() {
    let args = args_none();
    assert!(args.path.is_none());
}

#[test]
fn test_args_path_some() {
    let args = Args { path: Some(PathBuf::from("/data.toml")) };
    assert_eq!(args.path, Some(PathBuf::from("/data.toml")));
}

#[test]
fn test_resolve_file_not_directory_not_special() {
    // Test with a path that has no extension - treated as directory not found
    let args = args_with_path(PathBuf::from("/tmp/nonexistent_tv_test_no_ext"));
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Directory not found"),
        "Path without extension should be treated as directory: {err}"
    );
}

#[test]
fn test_resolve_missing_file_with_extension() {
    let args = args_with_path(PathBuf::from("/tmp/nonexistent_tv_test.toml"));
    let result = tv_lib::cli::resolve_paths(&args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("File not found") || err.contains("not found"),
        "Path with extension should mention file not found: {err}"
    );
}

#[test]
fn test_resolve_directory_handles_hidden_toml_files() {
    let dir = tempfile::tempdir().unwrap();
    let visible = dir.path().join("data.toml");
    let hidden = dir.path().join(".hidden.toml");
    std::fs::write(&visible, "[[items]]\n").unwrap();
    std::fs::write(&hidden, "[[items]]\n").unwrap();

    let args = args_with_path(dir.path().to_path_buf());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    // Both should be found since scan_directory_for_toml filters by .toml extension
    assert_eq!(result.files.len(), 2);
}

#[test]
fn test_resolve_single_file_not_requiring_toml_extension() {
    // resolve_paths should accept any file path, not just .toml
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("data.txt");
    std::fs::write(&file_path, "some content").unwrap();

    let args = args_with_path(file_path.clone());
    let result = tv_lib::cli::resolve_paths(&args).unwrap();

    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0], file_path);
}

#[test]
fn test_app_paths_clone() {
    let paths = AppPaths { files: vec![PathBuf::from("/test.toml")] };
    let cloned = paths.clone();
    assert_eq!(paths.files, cloned.files);
}

#[test]
fn test_app_paths_debug() {
    let paths = AppPaths { files: vec![PathBuf::from("/test.toml")] };
    let debug = format!("{paths:?}");
    assert!(debug.contains("test.toml"), "Debug should contain file path: {debug}");
}
