use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn find_rust_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip target directory and hidden directories
            if file_name.starts_with('.') || file_name == "target" {
                return false;
            }

            // Skip docs, benchmarks, tests, and old code directories
            if matches!(file_name, "docs" | "benchmarks" | "tests" | "old_tabula_cli") {
                return false;
            }

            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect()
}
