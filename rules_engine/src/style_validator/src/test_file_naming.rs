use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::violation::{StyleViolation, ViolationKind};

/// Checks that all test files under rules_engine/tests/ end in `_tests.rs`.
pub fn check_tests_directory(rules_engine_path: &Path) -> Result<Vec<StyleViolation>> {
    let mut violations = Vec::new();
    let tests_path = rules_engine_path.join("tests");

    for entry in WalkDir::new(&tests_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !file_name.ends_with(".rs") {
            continue;
        }

        // Skip non-test files that are allowed
        if file_name == "mod.rs" || file_name == "lib.rs" {
            continue;
        }

        // Skip files in src/ directories (these are library/helper code, not tests)
        if path.components().any(|c| c.as_os_str() == "src") {
            continue;
        }

        // Skip helper files (files with "helper" or "utils" in the name)
        if file_name.contains("helper") || file_name.contains("utils") {
            continue;
        }

        // Check that test files end in _tests.rs (not _test.rs)
        if file_name.ends_with("_test.rs") {
            violations.push(StyleViolation {
                file: path.to_path_buf(),
                line: 0,
                column: 0,
                kind: ViolationKind::TestFileNamingConvention,
                path_str: path
                    .strip_prefix(rules_engine_path)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            });
        }
    }

    Ok(violations)
}
