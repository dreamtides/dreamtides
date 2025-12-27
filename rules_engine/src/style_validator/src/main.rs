use std::path::Path;

use anyhow::Result;

mod file_scanner;
mod qualified_imports;
mod violation;

use file_scanner::find_rust_files;
use qualified_imports::check_file;

fn main() -> Result<()> {
    let rules_engine_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    println!("Checking Rust files in: {}", rules_engine_path.display());

    let rust_files = find_rust_files(rules_engine_path);
    println!("Found {} Rust files", rust_files.len());

    let mut all_violations = Vec::new();
    let style_validator_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    for file in &rust_files {
        // Skip the style_validator's own source files
        if file.starts_with(style_validator_path) {
            continue;
        }

        match check_file(file) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("Error checking {}: {}", file.display(), e);
            }
        }
    }

    if all_violations.is_empty() {
        println!("\n✓ No style violations found!");
        Ok(())
    } else {
        println!("\n✗ Found {} style violations:\n", all_violations.len());

        for violation in &all_violations {
            println!("{}:{}:{}", violation.file.display(), violation.line, violation.column,);
            println!("  → {}", violation.path_str);
            println!("  ✗ {}\n", violation.kind.description());
        }

        std::process::exit(1);
    }
}
