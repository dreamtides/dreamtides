use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::document::field_validation::{self, FieldError};
use crate::document::frontmatter_parser::{self, ParsedFrontmatter, UnknownKey};
use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;

/// A complete Lattice document with frontmatter and body.
#[derive(Debug, Clone)]
pub struct Document {
    /// Parsed and validated frontmatter.
    pub frontmatter: Frontmatter,
    /// The original raw YAML string for round-trip preservation.
    pub raw_yaml: String,
    /// The markdown body content.
    pub body: String,
    /// The 1-indexed line number where the body starts in the original file.
    pub body_start_line: usize,
}

/// Result of reading a document with additional diagnostics.
#[derive(Debug, Clone)]
pub struct ReadResult {
    /// The parsed document.
    pub document: Document,
    /// Unknown keys found in frontmatter (for linting).
    pub unknown_keys: Vec<UnknownKey>,
    /// Field validation errors (for linting).
    pub field_errors: Vec<FieldError>,
}

/// Reads a Lattice document from the filesystem.
///
/// This function:
/// 1. Reads the file content with UTF-8 BOM handling
/// 2. Parses the YAML frontmatter
/// 3. Returns the document with frontmatter and body
///
/// Validation is not performed; use `read_and_validate` for full validation.
pub fn read(path: &Path) -> Result<Document, LatticeError> {
    let content = read_file_content(path)?;
    let parsed = parse_content(&content, path)?;
    Ok(document_from_parsed(parsed))
}

/// Reads and validates a Lattice document from the filesystem.
///
/// This function:
/// 1. Reads the file content with UTF-8 BOM handling
/// 2. Parses the YAML frontmatter with unknown key detection
/// 3. Validates field values
/// 4. Returns the document with diagnostics
pub fn read_and_validate(path: &Path) -> Result<ReadResult, LatticeError> {
    let content = read_file_content(path)?;
    let (parsed, unknown_keys) = parse_content_with_diagnostics(&content, path)?;
    let document = document_from_parsed(parsed);
    let validation = field_validation::validate(&document.frontmatter, path);

    tracing::debug!(
        path = %path.display(),
        unknown_keys = unknown_keys.len(),
        field_errors = validation.errors.len(),
        "Document read and validated"
    );

    Ok(ReadResult { document, unknown_keys, field_errors: validation.errors })
}

/// Checks if a file appears to be a Lattice document.
///
/// Returns true if the file starts with `---` (possibly with BOM).
pub fn is_lattice_document(path: &Path) -> Result<bool, LatticeError> {
    let content = read_file_content(path)?;
    let trimmed = content.trim_start_matches('\u{feff}');
    Ok(trimmed.starts_with("---"))
}

/// Checks if file content appears to be a Lattice document.
pub fn content_is_lattice_document(content: &str) -> bool {
    let trimmed = content.trim_start_matches('\u{feff}');
    trimmed.starts_with("---")
}

/// Reads file content with UTF-8 encoding and BOM handling.
fn read_file_content(path: &Path) -> Result<String, LatticeError> {
    tracing::debug!(path = %path.display(), "Reading document file");

    let bytes = fs::read(path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            LatticeError::FileNotFound { path: path.to_path_buf() }
        } else if e.kind() == ErrorKind::PermissionDenied {
            LatticeError::PermissionDenied { path: path.to_path_buf() }
        } else {
            LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() }
        }
    })?;

    let content = String::from_utf8(bytes).map_err(|e| LatticeError::ReadError {
        path: path.to_path_buf(),
        reason: format!("invalid UTF-8 encoding: {e}"),
    })?;

    Ok(content)
}

/// Parses document content into frontmatter and body.
fn parse_content(content: &str, path: &Path) -> Result<ParsedFrontmatter, LatticeError> {
    frontmatter_parser::parse(content, path)
}

/// Parses document content with unknown key detection.
fn parse_content_with_diagnostics(
    content: &str,
    path: &Path,
) -> Result<(ParsedFrontmatter, Vec<UnknownKey>), LatticeError> {
    frontmatter_parser::parse_with_unknown_key_detection(content, path)
}

/// Converts ParsedFrontmatter to Document.
fn document_from_parsed(parsed: ParsedFrontmatter) -> Document {
    Document {
        frontmatter: parsed.frontmatter,
        raw_yaml: parsed.raw_yaml,
        body: parsed.body,
        body_start_line: parsed.body_start_line,
    }
}

impl Document {
    /// Returns true if this document is a task.
    pub fn is_task(&self) -> bool {
        self.frontmatter.is_task()
    }

    /// Returns true if this document is a knowledge base document.
    pub fn is_knowledge_base(&self) -> bool {
        self.frontmatter.is_knowledge_base()
    }

    /// Returns the document's Lattice ID as a string.
    pub fn id_str(&self) -> &str {
        self.frontmatter.lattice_id.as_str()
    }
}

impl ReadResult {
    /// Returns true if the document has no validation issues.
    pub fn is_clean(&self) -> bool {
        self.unknown_keys.is_empty() && self.field_errors.is_empty()
    }
}
