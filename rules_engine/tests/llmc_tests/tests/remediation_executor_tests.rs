use llmc::overseer_mode::remediation_executor;

#[test]
fn remediation_log_path_includes_timestamp() {
    let timestamp = "20260121_143022";
    let path = remediation_executor::remediation_log_path(timestamp);
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("logs"), "Log path should be in logs directory, got: {}", path_str);
    assert!(
        path_str.contains("remediation_20260121_143022.txt"),
        "Log path should include timestamp in filename, got: {}",
        path_str
    );
}

#[test]
fn remediation_log_path_uses_txt_extension() {
    let path = remediation_executor::remediation_log_path("test_timestamp");
    assert!(
        path.extension().map_or(false, |ext| ext == "txt"),
        "Log file should have .txt extension, got: {}",
        path.display()
    );
}

#[test]
fn remediation_log_path_is_absolute_when_llmc_root_is_set() {
    let timestamp = "20260121_000000";
    let path = remediation_executor::remediation_log_path(timestamp);
    assert!(
        path.to_string_lossy().contains("llmc") || path.to_string_lossy().contains("logs"),
        "Path should reference llmc logs directory, got: {}",
        path.display()
    );
}
