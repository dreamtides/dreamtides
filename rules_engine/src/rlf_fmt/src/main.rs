use std::fs;
use std::path::{Path, PathBuf};

use rlf::formatter;

const MAX_WIDTH: usize = 100;
const MACRO_INDENT: &str = "    ";

fn main() {
    let manifest_dir = find_manifest_dir();
    let mut changed_count = 0;

    // Format all .rlf locale files
    let locales_dir = manifest_dir.join("src/strings/locales");
    if locales_dir.is_dir() {
        for entry in fs::read_dir(&locales_dir).expect("Failed to read locales directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rlf") && format_rlf_file(&path) {
                changed_count += 1;
            }
        }
    }

    // Format strings.rs containing rlf! macro
    let strings_path = manifest_dir.join("src/strings/src/strings.rs");
    if strings_path.is_file() && format_strings_rs(&strings_path) {
        changed_count += 1;
    }

    if changed_count > 0 {
        println!("RLF formatted {changed_count} file(s)");
    } else {
        println!("RLF files already formatted");
    }
}

/// Formats a `.rlf` locale file in place.
fn format_rlf_file(path: &Path) -> bool {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let formatted = formatter::format_file(&content, MAX_WIDTH);
    if formatted != content {
        fs::write(path, &formatted)
            .unwrap_or_else(|e| panic!("Failed to write {}: {e}", path.display()));
        println!("  Formatted {}", path.display());
        true
    } else {
        false
    }
}

/// Formats the `rlf::rlf! { ... }` macro body within a Rust source file.
fn format_strings_rs(path: &Path) -> bool {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

    let Some((before, body, after)) = extract_rlf_macro_body(&content) else {
        eprintln!("  Warning: no rlf::rlf! macro found in {}", path.display());
        return false;
    };

    // Trim leading/trailing whitespace from body to avoid accumulating blank lines
    let trimmed_body = body.trim();
    let formatted_body = formatter::format_definitions(trimmed_body, MACRO_INDENT, MAX_WIDTH);

    let mut result = String::with_capacity(content.len());
    result.push_str(before);
    result.push_str("rlf::rlf! {\n");
    result.push_str(&formatted_body);
    result.push_str("}\n");
    // Preserve non-whitespace content after the macro, normalize trailing newlines
    let after_trimmed = after.trim_start_matches('\n');
    if !after_trimmed.is_empty() {
        result.push_str(after_trimmed);
    }

    if result != content {
        fs::write(path, &result)
            .unwrap_or_else(|e| panic!("Failed to write {}: {e}", path.display()));
        println!("  Formatted {}", path.display());
        true
    } else {
        false
    }
}

/// Extracts the body of the `rlf::rlf! { ... }` macro from a Rust source
/// file. Returns `(before_macro, body_content, after_macro)`.
fn extract_rlf_macro_body(content: &str) -> Option<(&str, String, &str)> {
    // Find `rlf::rlf!` followed by `{`
    let macro_pattern = "rlf::rlf!";
    let macro_start = content.find(macro_pattern)?;

    // Find the opening `{` after the macro invocation
    let after_macro = &content[macro_start + macro_pattern.len()..];
    let brace_offset = after_macro.find('{')?;
    let body_start = macro_start + macro_pattern.len() + brace_offset + 1;

    // Track brace depth to find matching `}`, respecting strings and comments.
    // Use char_indices to get correct byte offsets for multi-byte UTF-8.
    let body_slice = &content[body_start..];
    let mut depth = 1i32;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut char_iter = body_slice.char_indices().peekable();

    while let Some((byte_offset, ch)) = char_iter.next() {
        if depth == 0 {
            break;
        }

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        // Handle escape sequences in strings
        if ch == '\\' && in_string {
            char_iter.next(); // skip escaped char
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            continue;
        }

        if !in_string {
            // Check for line comments
            if ch == '/' {
                if let Some(&(_, '/')) = char_iter.peek() {
                    in_line_comment = true;
                    char_iter.next();
                    continue;
                }
            }

            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let body = &content[body_start..body_start + byte_offset];
                    let before = &content[..macro_start];
                    let after = &content[body_start + byte_offset + 1..];
                    return Some((before, body.to_string(), after));
                }
            }
        }
    }

    None
}

/// Finds the rules_engine directory by walking up from the binary's manifest.
fn find_manifest_dir() -> PathBuf {
    // Try relative to current directory first
    let candidates = [PathBuf::from("rules_engine"), PathBuf::from(".")];

    for candidate in &candidates {
        if candidate.join("src/strings/locales").is_dir() {
            return candidate.clone();
        }
    }

    // Try finding via CARGO_MANIFEST_DIR at compile time
    let manifest = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest);
    // Go up from src/rlf_fmt to rules_engine
    if let Some(parent) = manifest_path.parent().and_then(|p| p.parent()) {
        if parent.join("src/strings/locales").is_dir() {
            return parent.to_path_buf();
        }
    }

    panic!("Could not find rules_engine directory. Run from the project root.");
}
