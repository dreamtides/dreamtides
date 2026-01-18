use std::fs;
use std::path::Path;

use tracing::{debug, info, warn};

use crate::document::{document_reader, document_writer};
use crate::error::error_types::LatticeError;
use crate::format::text_wrapper::{self, WrapConfig};
use crate::lint::autofix_engine;

/// Default line width for formatting.
pub const DEFAULT_LINE_WIDTH: usize = 80;

/// Configuration for markdown formatting operations.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Maximum line width for text wrapping.
    pub line_width: usize,
    /// Whether to run in dry-run mode (check without writing).
    pub dry_run: bool,
    /// Whether to clean whitespace (trailing spaces, multiple blanks).
    pub clean_whitespace: bool,
    /// Whether to normalize headers (ATX conversion, blank lines).
    pub normalize_headers: bool,
    /// Whether to normalize lists (markers, blank lines).
    pub normalize_lists: bool,
    /// Whether to wrap text at line_width.
    pub wrap_text: bool,
    /// Whether to ensure final newline.
    pub ensure_final_newline: bool,
}

/// Result of formatting a single document.
#[derive(Debug)]
pub enum FormatResult {
    /// Document was modified.
    Modified,
    /// Document was unchanged.
    Unchanged,
    /// Document was skipped (dry-run mode, would have been modified).
    WouldModify,
}

/// Summary of formatting multiple documents.
#[derive(Debug, Default)]
pub struct FormatSummary {
    /// Number of files that were formatted (modified).
    pub files_formatted: usize,
    /// Number of files that were unchanged.
    pub files_unchanged: usize,
    /// Number of files that would be modified (dry-run mode).
    pub files_would_modify: usize,
    /// Errors encountered during formatting.
    pub errors: Vec<FormatError>,
}

/// An error encountered while formatting a document.
#[derive(Debug)]
pub struct FormatError {
    /// Path to the file that failed.
    pub path: std::path::PathBuf,
    /// The error that occurred.
    pub error: LatticeError,
}

/// Formats markdown content by applying all enabled operations.
///
/// Operations are applied in the following order:
/// 1. Whitespace cleaning (trailing spaces, multiple blank lines)
/// 2. Header normalization (setext to ATX, blank line spacing)
/// 3. List normalization (markers to dashes, blank line spacing)
/// 4. Text wrapping at configured width
/// 5. Final newline enforcement
///
/// Returns the formatted content and whether any changes were made.
pub fn format_content(content: &str, config: &FormatConfig) -> (String, bool) {
    let original = content.to_string();
    let mut result = content.to_string();

    if config.clean_whitespace {
        result = autofix_engine::fix_trailing_whitespace(&result);
        result = autofix_engine::fix_multiple_blank_lines(&result);
    }

    if config.normalize_headers {
        result = autofix_engine::fix_setext_headers(&result);
        result = autofix_engine::fix_heading_blank_lines(&result);
    }

    if config.normalize_lists {
        result = autofix_engine::fix_list_markers(&result);
        result = autofix_engine::fix_list_blank_lines(&result);
    }

    if config.wrap_text {
        let wrap_config = WrapConfig::new(config.line_width);
        let wrap_result = text_wrapper::wrap(&result, &wrap_config);
        result = wrap_result.content;
    }

    if config.ensure_final_newline {
        result = autofix_engine::fix_missing_final_newline(&result);
    }

    let modified = result != original;
    (result, modified)
}

/// Formats a single document at the given path.
///
/// Reads the document, applies formatting to the body, and writes atomically
/// if changes were made. In dry-run mode, reports whether changes would be made
/// without writing.
///
/// Returns whether the document was modified (or would be modified in dry-run).
pub fn format_document(path: &Path, config: &FormatConfig) -> Result<FormatResult, LatticeError> {
    debug!(path = %path.display(), "Formatting document");

    let document = document_reader::read(path)?;
    let (formatted_body, modified) = format_content(&document.body, config);

    if !modified {
        debug!(path = %path.display(), "Document unchanged");
        return Ok(FormatResult::Unchanged);
    }

    if config.dry_run {
        info!(path = %path.display(), "Document would be modified (dry-run)");
        return Ok(FormatResult::WouldModify);
    }

    document_writer::update_body(path, &formatted_body, &document_writer::WriteOptions::default())?;

    info!(path = %path.display(), "Document formatted");
    Ok(FormatResult::Modified)
}

/// Formats all markdown documents in a directory.
///
/// Recursively finds all `.md` files that are valid Lattice documents and
/// formats them according to the configuration. Returns a summary of the
/// operation including counts of formatted, unchanged, and errored files.
pub fn format_directory(path: &Path, config: &FormatConfig) -> Result<FormatSummary, LatticeError> {
    debug!(path = %path.display(), "Formatting directory");

    if !path.is_dir() {
        return Err(LatticeError::InvalidPath {
            path: path.to_path_buf(),
            reason: "path is not a directory".to_string(),
        });
    }

    let mut summary = FormatSummary::default();
    format_directory_recursive(path, config, &mut summary)?;

    info!(
        path = %path.display(),
        formatted = summary.files_formatted,
        unchanged = summary.files_unchanged,
        would_modify = summary.files_would_modify,
        errors = summary.errors.len(),
        "Directory formatting complete"
    );

    Ok(summary)
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: DEFAULT_LINE_WIDTH,
            dry_run: false,
            clean_whitespace: true,
            normalize_headers: true,
            normalize_lists: true,
            wrap_text: true,
            ensure_final_newline: true,
        }
    }
}

impl FormatConfig {
    /// Creates a new configuration with the specified line width.
    pub fn new(line_width: usize) -> Self {
        Self { line_width, ..Self::default() }
    }

    /// Creates a configuration for dry-run mode.
    pub fn dry_run() -> Self {
        Self { dry_run: true, ..Self::default() }
    }

    /// Sets the line width.
    pub fn with_line_width(mut self, width: usize) -> Self {
        self.line_width = width;
        self
    }

    /// Sets dry-run mode.
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl FormatSummary {
    /// Returns the total number of files processed.
    pub fn total_files(&self) -> usize {
        self.files_formatted + self.files_unchanged + self.files_would_modify + self.errors.len()
    }

    /// Returns true if any errors occurred.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn format_directory_recursive(
    dir: &Path,
    config: &FormatConfig,
    summary: &mut FormatSummary,
) -> Result<(), LatticeError> {
    let entries = fs::read_dir(dir).map_err(|e| LatticeError::ReadError {
        path: dir.to_path_buf(),
        reason: format!("failed to read directory: {e}"),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| LatticeError::ReadError {
            path: dir.to_path_buf(),
            reason: format!("failed to read directory entry: {e}"),
        })?;

        let path = entry.path();

        if path.is_dir() {
            if !is_hidden_directory(&path) {
                format_directory_recursive(&path, config, summary)?;
            }
            continue;
        }

        if !is_markdown_file(&path) {
            continue;
        }

        match document_reader::is_lattice_document(&path) {
            Ok(false) => {
                debug!(path = %path.display(), "Skipping non-Lattice document");
                continue;
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to check document type");
                summary.errors.push(FormatError { path, error: e });
                continue;
            }
            Ok(true) => {}
        }

        match format_document(&path, config) {
            Ok(FormatResult::Modified) => summary.files_formatted += 1,
            Ok(FormatResult::Unchanged) => summary.files_unchanged += 1,
            Ok(FormatResult::WouldModify) => summary.files_would_modify += 1,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to format document");
                summary.errors.push(FormatError { path, error: e });
            }
        }
    }

    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}

fn is_hidden_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.'))
}
