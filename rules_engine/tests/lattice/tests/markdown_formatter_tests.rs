//! Tests for the markdown formatter module.

use std::fs;
use std::io::Write as IoWrite;

use lattice::format::markdown_formatter::{
    FormatConfig, FormatResult, format_content, format_directory, format_document,
};
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

fn create_lattice_document(dir: &TempDir, name: &str, body: &str) -> std::path::PathBuf {
    let content = format!(
        r#"---
lattice-id: LABCDT
name: {name}
description: Test document
---

{body}
"#,
        name = name.replace(".md", "").replace("_", "-"),
        body = body
    );
    create_temp_file(dir, name, &content)
}

// =============================================================================
// format_content Tests
// =============================================================================

#[test]
fn format_content_no_changes() {
    let config = FormatConfig::default();
    let content = "Simple text.\n";
    let (result, modified) = format_content(content, &config);
    assert!(!modified, "Content should be unchanged");
    assert_eq!(result, content);
}

#[test]
fn format_content_removes_trailing_whitespace() {
    let config = FormatConfig::default();
    let content = "Line with trailing spaces.   \n";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Content should be modified");
    assert!(!result.contains("   \n"), "Trailing spaces should be removed");
}

#[test]
fn format_content_collapses_multiple_blank_lines() {
    let config = FormatConfig::default();
    let content = "First paragraph.\n\n\n\nSecond paragraph.\n";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Content should be modified");
    assert!(!result.contains("\n\n\n"), "Multiple blank lines should be collapsed");
}

#[test]
fn format_content_converts_setext_to_atx() {
    let config = FormatConfig::default();
    let content = "Heading\n=======\n";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Content should be modified");
    assert!(result.contains("# Heading"), "Setext heading should be ATX");
}

#[test]
fn format_content_normalizes_list_markers() {
    let config = FormatConfig::default();
    let content = "* Item one\n* Item two\n";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Content should be modified");
    assert!(result.contains("- Item one"), "Asterisk marker should become dash");
}

#[test]
fn format_content_adds_final_newline() {
    let config = FormatConfig::default();
    let content = "No final newline";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Content should be modified");
    assert!(result.ends_with('\n'), "Should have final newline");
}

#[test]
fn format_content_respects_disabled_operations() {
    let mut config = FormatConfig::default();
    config.clean_whitespace = false;
    config.normalize_headers = false;
    config.normalize_lists = false;
    config.wrap_text = false;
    config.ensure_final_newline = false;

    let content = "Line with trailing spaces.   ";
    let (result, modified) = format_content(content, &config);
    assert!(!modified, "No operations enabled, should be unchanged");
    assert_eq!(result, content);
}

#[test]
fn format_content_wraps_long_lines() {
    let mut config = FormatConfig::default();
    config.line_width = 40;
    let content =
        "This is a very long line that exceeds the configured line width and should be wrapped.\n";
    let (result, modified) = format_content(content, &config);
    assert!(modified, "Long line should be wrapped");
    for line in result.lines() {
        if !line.is_empty() {
            assert!(line.len() <= 40, "Line should be at most 40 chars: '{line}' ({})", line.len());
        }
    }
}

// =============================================================================
// format_document Tests
// =============================================================================

#[test]
fn format_document_unchanged() {
    let dir = TempDir::new().unwrap();
    let path = create_lattice_document(&dir, "unchanged.md", "Simple text.");
    let config = FormatConfig::default();

    let result = format_document(&path, &config).expect("Should format document");
    assert!(matches!(result, FormatResult::Unchanged), "Document should be unchanged");
}

#[test]
fn format_document_modifies_file() {
    let dir = TempDir::new().unwrap();
    let path = create_lattice_document(&dir, "to_modify.md", "Text with trailing spaces.   ");
    let config = FormatConfig::default();

    let result = format_document(&path, &config).expect("Should format document");
    assert!(matches!(result, FormatResult::Modified), "Document should be modified");

    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.contains("   \n"), "Trailing spaces should be removed");
}

#[test]
fn format_document_dry_run_does_not_modify() {
    let dir = TempDir::new().unwrap();
    let body = "Text with trailing spaces.   ";
    let path = create_lattice_document(&dir, "dry_run.md", body);
    let original_content = fs::read_to_string(&path).unwrap();
    let config = FormatConfig::dry_run();

    let result = format_document(&path, &config).expect("Should check document");
    assert!(matches!(result, FormatResult::WouldModify), "Should report would modify");

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, original_content, "File should not be modified in dry-run");
}

#[test]
fn format_document_not_found() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.md");
    let config = FormatConfig::default();

    let result = format_document(&path, &config);
    assert!(result.is_err(), "Should fail for nonexistent file");
}

// =============================================================================
// format_directory Tests
// =============================================================================

#[test]
fn format_directory_empty() {
    let dir = TempDir::new().unwrap();
    let config = FormatConfig::default();

    let summary = format_directory(dir.path(), &config).expect("Should format empty directory");
    assert_eq!(summary.files_formatted, 0);
    assert_eq!(summary.files_unchanged, 0);
    assert_eq!(summary.total_files(), 0);
}

#[test]
fn format_directory_with_documents() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "doc1.md", "Clean content.");
    create_lattice_document(&dir, "doc2.md", "Dirty content.   ");
    let config = FormatConfig::default();

    let summary = format_directory(dir.path(), &config).expect("Should format directory");
    assert_eq!(summary.total_files(), 2);
    assert!(summary.files_formatted >= 1, "At least one file should be formatted");
}

#[test]
fn format_directory_skips_hidden() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "visible.md", "Visible content.");

    let hidden_dir = dir.path().join(".hidden");
    fs::create_dir(&hidden_dir).unwrap();
    let hidden_file = hidden_dir.join("hidden.md");
    let hidden_content = r#"---
lattice-id: LHIDDN
name: hidden
description: Hidden document
---

Hidden content.
"#;
    fs::write(&hidden_file, hidden_content).unwrap();

    let config = FormatConfig::default();
    let summary = format_directory(dir.path(), &config).expect("Should format directory");
    assert_eq!(summary.total_files(), 1, "Should only process visible files");
}

#[test]
fn format_directory_skips_non_lattice() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "lattice.md", "Lattice content.");
    create_temp_file(&dir, "plain.md", "# Plain markdown without frontmatter");
    let config = FormatConfig::default();

    let summary = format_directory(dir.path(), &config).expect("Should format directory");
    assert_eq!(summary.total_files(), 1, "Should only process Lattice documents");
}

#[test]
fn format_directory_recursive() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "root.md", "Root content.");
    create_lattice_document(&dir, "subdir/nested.md", "Nested content.");
    let config = FormatConfig::default();

    let summary =
        format_directory(dir.path(), &config).expect("Should format directory recursively");
    assert_eq!(summary.total_files(), 2, "Should process files in subdirectories");
}

#[test]
fn format_directory_dry_run() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "dirty.md", "Dirty content.   ");
    let config = FormatConfig::dry_run();

    let summary = format_directory(dir.path(), &config).expect("Should check directory");
    assert_eq!(summary.files_formatted, 0, "No files should be formatted in dry-run");
    assert_eq!(summary.files_would_modify, 1, "Should report would modify");
}

#[test]
fn format_directory_not_a_directory() {
    let dir = TempDir::new().unwrap();
    let file_path = create_temp_file(&dir, "file.txt", "content");
    let config = FormatConfig::default();

    let result = format_directory(&file_path, &config);
    assert!(result.is_err(), "Should fail for non-directory path");
}

// =============================================================================
// FormatConfig Tests
// =============================================================================

#[test]
fn format_config_default() {
    let config = FormatConfig::default();
    assert_eq!(config.line_width, 80);
    assert!(!config.dry_run);
    assert!(config.clean_whitespace);
    assert!(config.normalize_headers);
    assert!(config.normalize_lists);
    assert!(config.wrap_text);
    assert!(config.ensure_final_newline);
}

#[test]
fn format_config_new() {
    let config = FormatConfig::new(120);
    assert_eq!(config.line_width, 120);
    assert!(!config.dry_run);
}

#[test]
fn format_config_dry_run() {
    let config = FormatConfig::dry_run();
    assert!(config.dry_run);
    assert_eq!(config.line_width, 80);
}

#[test]
fn format_config_builder_pattern() {
    let config = FormatConfig::default().with_line_width(100).with_dry_run(true);
    assert_eq!(config.line_width, 100);
    assert!(config.dry_run);
}

// =============================================================================
// FormatSummary Tests
// =============================================================================

#[test]
fn format_summary_total_files() {
    let dir = TempDir::new().unwrap();
    create_lattice_document(&dir, "doc1.md", "Content one.");
    create_lattice_document(&dir, "doc2.md", "Content two.");
    create_lattice_document(&dir, "doc3.md", "Content three.");
    let config = FormatConfig::default();

    let summary = format_directory(dir.path(), &config).unwrap();
    assert_eq!(summary.total_files(), 3);
}

#[test]
fn format_summary_has_errors() {
    let dir = TempDir::new().unwrap();
    let config = FormatConfig::default();

    let summary = format_directory(dir.path(), &config).unwrap();
    assert!(!summary.has_errors(), "Empty directory should have no errors");
}

// =============================================================================
// Consecutive Blank Lines Regression Tests
// =============================================================================

#[test]
fn format_content_consecutive_headings_no_double_blanks() {
    let config = FormatConfig::default();
    let content = "## Heading 1\n## Heading 2\n";
    let (result, _modified) = format_content(content, &config);
    assert!(
        !result.contains("\n\n\n"),
        "Consecutive headings should not produce consecutive blank lines.\nResult:\n{result}"
    );
}

#[test]
fn format_content_heading_followed_by_list_no_double_blanks() {
    let config = FormatConfig::default();
    let content = "## Heading\n- Item 1\n- Item 2\n";
    let (result, _modified) = format_content(content, &config);
    assert!(
        !result.contains("\n\n\n"),
        "Heading followed by list should not produce consecutive blank lines.\nResult:\n{result}"
    );
}

#[test]
fn format_content_list_followed_by_heading_no_double_blanks() {
    let config = FormatConfig::default();
    let content = "- Item 1\n- Item 2\n## Heading\n";
    let (result, _modified) = format_content(content, &config);
    assert!(
        !result.contains("\n\n\n"),
        "List followed by heading should not produce consecutive blank lines.\nResult:\n{result}"
    );
}

#[test]
fn format_content_multiple_blank_lines_after_formatting_still_collapsed() {
    let config = FormatConfig::default();
    let content = "## Heading 1\n\n\n## Heading 2\n";
    let (result, _modified) = format_content(content, &config);
    assert!(
        !result.contains("\n\n\n"),
        "Multiple blank lines should remain collapsed after all formatting.\nResult:\n{result}"
    );
}
