use std::path::Path;

use anyhow::Result;

mod direct_function_imports;
mod file_scanner;
mod mod_lib_files;
mod qualified_imports;
mod violation;

use file_scanner::find_rust_files;

fn main() -> Result<()> {
    let rules_engine_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    println!("Checking Rust files in: {}", rules_engine_path.display());

    let rust_files = find_rust_files(rules_engine_path);
    println!("Found {} Rust files", rust_files.len());

    let mut all_violations = Vec::new();
    let style_validator_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Filter out style_validator's own files
    let rust_files: Vec<_> =
        rust_files.into_iter().filter(|file| !file.starts_with(style_validator_path)).collect();

    // Run per-file checks
    for file in &rust_files {
        match qualified_imports::check_file(file) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("Error checking {}: {}", file.display(), e);
            }
        }

        match mod_lib_files::check_file(file) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("Error checking {}: {}", file.display(), e);
            }
        }
    }

    // Run cross-file checks
    match direct_function_imports::check_all_files(&rust_files, rules_engine_path) {
        Ok(violations) => {
            all_violations.extend(violations);
        }
        Err(e) => {
            eprintln!("Error checking direct function imports: {e}");
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
