use tv_lib::logging::json_logger;

fn extract_date_part(file_name: &str) -> &str {
    let without_prefix = file_name.strip_prefix("tv_").expect("Should start with tv_");
    without_prefix.strip_suffix(".jsonl").expect("Should end with .jsonl")
}

#[test]
fn test_log_file_path_contains_tv_logs_directory() {
    let path = json_logger::log_file_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("tv") && path_str.contains("logs"),
        "Log file path should include tv/logs directory: {path_str}"
    );
}

#[test]
fn test_log_file_path_has_jsonl_extension() {
    let path = json_logger::log_file_path();
    let extension = path.extension().expect("Log file path should have an extension");
    assert_eq!(extension, "jsonl", "Log file should have .jsonl extension");
}

#[test]
fn test_log_file_path_contains_date_stamp() {
    let path = json_logger::log_file_path();
    let file_name = path.file_name().expect("Should have file name").to_string_lossy();
    assert!(file_name.starts_with("tv_"), "Log file should start with tv_ prefix: {file_name}");
    let date_part = extract_date_part(&file_name);
    assert_eq!(date_part.len(), 10, "Date part should be YYYY-MM-DD format: {date_part}");
    assert_eq!(
        date_part.chars().filter(|c| *c == '-').count(),
        2,
        "Date part should contain two dashes: {date_part}"
    );
}

#[test]
fn test_log_file_path_is_absolute() {
    let path = json_logger::log_file_path();
    assert!(path.is_absolute(), "Log file path should be absolute: {}", path.display());
}

#[test]
fn test_log_file_path_is_deterministic() {
    let path1 = json_logger::log_file_path();
    let path2 = json_logger::log_file_path();
    assert_eq!(path1, path2, "Successive calls should return the same path within the same day");
}

#[test]
fn test_log_file_path_date_format_valid() {
    let path = json_logger::log_file_path();
    let file_name = path.file_name().expect("Should have file name").to_string_lossy();
    let date_part = extract_date_part(&file_name);
    let parts: Vec<&str> = date_part.split('-').collect();
    assert_eq!(parts.len(), 3, "Date should have three parts: {date_part}");
    let year: u32 = parts[0].parse().expect("Year should be numeric");
    let month: u32 = parts[1].parse().expect("Month should be numeric");
    let day: u32 = parts[2].parse().expect("Day should be numeric");
    assert!(year >= 2024 && year <= 2100, "Year should be reasonable: {year}");
    assert!((1..=12).contains(&month), "Month should be 1-12: {month}");
    assert!((1..=31).contains(&day), "Day should be 1-31: {day}");
}

#[test]
fn test_log_file_path_parent_is_logs_directory() {
    let path = json_logger::log_file_path();
    let parent = path.parent().expect("Should have parent directory");
    let parent_name = parent.file_name().expect("Parent should have name").to_string_lossy();
    assert_eq!(parent_name.as_ref(), "logs", "Parent directory should be 'logs'");
}

#[test]
fn test_initialize_jsonl_mode() {
    json_logger::initialize(true);
}
