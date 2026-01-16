use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use crate::violation::{StyleViolation, ViolationKind};

/// Checks for 'tests' directories under rules_engine/src/ which should instead
/// be placed in rules_engine/tests/.
pub fn check_src_directory(rules_engine_path: &Path) -> Result<Vec<StyleViolation>> {
    let mut violations = Vec::new();
    let src_path = rules_engine_path.join("src");

    for entry in WalkDir::new(&src_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() && path.file_name().map(|n| n == "tests").unwrap_or(false) {
            violations.push(StyleViolation {
                file: path.to_path_buf(),
                line: 0,
                column: 0,
                kind: ViolationKind::TestsDirectoryInSrc,
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
