use std::fs;
use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::commands::generate;

/// Verifies that generated artifacts match their source files.
pub fn check() -> Result<ExitCode> {
    let tabula_dir = generate::tabula_source_dir();
    let output_dir = generate::default_output_dir();
    let mut has_mismatch = false;

    // Check test_card.rs
    let expected = generate::generate_test_card_string(&tabula_dir)?;
    let actual_path = output_dir.join("test_card.rs");
    let actual = fs::read_to_string(&actual_path)
        .with_context(|| format!("Failed to read {}", actual_path.display()))?;
    if expected != actual {
        eprintln!("Mismatch: test_card.rs");
        print_diff(&actual, &expected);
        has_mismatch = true;
    }

    // Check card_lists.rs
    let expected = generate::generate_card_lists_string(&tabula_dir)?;
    let actual_path = output_dir.join("card_lists.rs");
    let actual = fs::read_to_string(&actual_path)
        .with_context(|| format!("Failed to read {}", actual_path.display()))?;
    if expected != actual {
        eprintln!("Mismatch: card_lists.rs");
        print_diff(&actual, &expected);
        has_mismatch = true;
    }

    // Check parsed_abilities.json (compare as JSON values)
    let expected = generate::generate_parsed_abilities_string(&tabula_dir)?;
    let actual_path = tabula_dir.join("parsed_abilities.json");
    let actual = fs::read_to_string(&actual_path)
        .with_context(|| format!("Failed to read {}", actual_path.display()))?;
    let expected_json: serde_json::Value = serde_json::from_str(&expected)
        .context("Failed to parse expected parsed_abilities.json")?;
    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).context("Failed to parse actual parsed_abilities.json")?;
    if expected_json != actual_json {
        eprintln!("Mismatch: parsed_abilities.json");
        print_diff(&actual, &expected);
        has_mismatch = true;
    }

    if has_mismatch {
        eprintln!("\nRun `tabula generate` to regenerate files.");
        Ok(ExitCode::FAILURE)
    } else {
        println!("All generated files are up to date.");
        Ok(ExitCode::SUCCESS)
    }
}

/// Prints a simple diff showing the first difference between two strings.
fn print_diff(actual: &str, expected: &str) {
    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();

    for (i, (a, e)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
        if a != e {
            eprintln!("  First difference at line {}:", i + 1);
            eprintln!("    actual:   {a}");
            eprintln!("    expected: {e}");
            return;
        }
    }

    if actual_lines.len() != expected_lines.len() {
        eprintln!(
            "  Line count differs: actual={}, expected={}",
            actual_lines.len(),
            expected_lines.len()
        );
    }
}
