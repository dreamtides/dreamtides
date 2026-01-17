use std::fs;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use tempfile::NamedTempFile;

use crate::document::document_reader::{self, Document};
use crate::document::frontmatter_parser;
use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;

/// Options for writing a document.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Whether to update the `updated-at` timestamp.
    pub update_timestamp: bool,
    /// Whether to create parent directories if they don't exist.
    pub create_parents: bool,
}

/// Writes a document to the filesystem atomically.
///
/// Uses a write-to-temp-then-rename strategy to prevent corruption on crash.
/// The document is formatted as YAML frontmatter + separator + body.
pub fn write(document: &Document, path: &Path, options: &WriteOptions) -> Result<(), LatticeError> {
    let frontmatter = prepare_frontmatter(&document.frontmatter, options);
    let content = format_document_content(&frontmatter, &document.body)?;
    atomic_write(path, &content, options)?;

    tracing::info!(
        path = %path.display(),
        id = %frontmatter.lattice_id,
        "Document written"
    );

    Ok(())
}

/// Writes a new document to the filesystem atomically.
///
/// Convenience function that creates a Document from frontmatter and body.
pub fn write_new(
    frontmatter: &Frontmatter,
    body: &str,
    path: &Path,
    options: &WriteOptions,
) -> Result<(), LatticeError> {
    let frontmatter = prepare_frontmatter(frontmatter, options);
    let content = format_document_content(&frontmatter, body)?;
    atomic_write(path, &content, options)?;

    tracing::info!(
        path = %path.display(),
        id = %frontmatter.lattice_id,
        "New document created"
    );

    Ok(())
}

/// Writes raw content to a path atomically.
///
/// Useful when the caller has already formatted the document content.
pub fn write_raw(path: &Path, content: &str, options: &WriteOptions) -> Result<(), LatticeError> {
    atomic_write(path, content, options)?;
    tracing::debug!(path = %path.display(), "Raw content written");
    Ok(())
}

/// Updates only the frontmatter of an existing document.
///
/// Preserves the body content exactly as-is.
pub fn update_frontmatter(
    path: &Path,
    frontmatter: &Frontmatter,
    options: &WriteOptions,
) -> Result<(), LatticeError> {
    let document = document_reader::read(path)?;
    let frontmatter = prepare_frontmatter(frontmatter, options);
    let content = format_document_content(&frontmatter, &document.body)?;
    atomic_write(path, &content, options)?;

    tracing::debug!(
        path = %path.display(),
        id = %frontmatter.lattice_id,
        "Frontmatter updated"
    );

    Ok(())
}

/// Updates only the body of an existing document.
///
/// Preserves the frontmatter exactly as-is, optionally updating the timestamp.
pub fn update_body(path: &Path, body: &str, options: &WriteOptions) -> Result<(), LatticeError> {
    let document = document_reader::read(path)?;
    let frontmatter = prepare_frontmatter(&document.frontmatter, options);
    let content = format_document_content(&frontmatter, body)?;
    atomic_write(path, &content, options)?;

    tracing::debug!(
        path = %path.display(),
        id = %frontmatter.lattice_id,
        "Body updated"
    );

    Ok(())
}

/// Prepares frontmatter for writing, optionally updating timestamp.
fn prepare_frontmatter(frontmatter: &Frontmatter, options: &WriteOptions) -> Frontmatter {
    let mut result = frontmatter.clone();

    if options.update_timestamp {
        result.updated_at = Some(Utc::now());
    }

    result
}

/// Formats a document as frontmatter + separator + body.
fn format_document_content(frontmatter: &Frontmatter, body: &str) -> Result<String, LatticeError> {
    frontmatter_parser::format_document(frontmatter, body)
}

/// Performs atomic write using temp file and rename.
fn atomic_write(path: &Path, content: &str, options: &WriteOptions) -> Result<(), LatticeError> {
    if options.create_parents
        && let Some(parent) = path.parent()
    {
        fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: format!("failed to create parent directories: {e}"),
        })?;
    }

    let parent = path.parent().unwrap_or(Path::new("."));
    let temp_file = NamedTempFile::new_in(parent).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("failed to create temp file: {e}"),
    })?;

    temp_file.as_file().write_all(content.as_bytes()).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("failed to write content: {e}"),
    })?;

    temp_file.as_file().sync_all().map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("failed to sync file: {e}"),
    })?;

    temp_file.persist(path).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("failed to rename temp file: {e}"),
    })?;

    tracing::trace!(path = %path.display(), "Atomic write completed");

    Ok(())
}

impl WriteOptions {
    /// Creates options with timestamp update enabled.
    pub fn with_timestamp() -> Self {
        Self { update_timestamp: true, ..Default::default() }
    }

    /// Creates options with parent directory creation enabled.
    pub fn with_parents() -> Self {
        Self { create_parents: true, ..Default::default() }
    }

    /// Creates options with both timestamp update and parent creation.
    pub fn with_timestamp_and_parents() -> Self {
        Self { update_timestamp: true, create_parents: true }
    }
}
