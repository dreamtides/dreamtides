use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_queries;

/// A successfully resolved link with the target's current file path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLink {
    /// The target document's Lattice ID.
    pub target_id: LatticeId,
    /// Relative path from the source document to the target document.
    pub relative_path: String,
    /// Full link URL with path and fragment (e.g.,
    /// `../design/system.md#LJCQ2X`).
    pub link_url: String,
}

/// Reason why a link could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnresolvedReason {
    /// The target ID was not found in the index.
    TargetNotFound,
    /// The target document exists but has an invalid path.
    InvalidTargetPath { path: String },
}

/// An unresolved link with error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedLink {
    /// The target ID that could not be resolved.
    pub target_id: LatticeId,
    /// The reason resolution failed.
    pub reason: UnresolvedReason,
}

/// Result of attempting to resolve a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkResolution {
    /// Link was successfully resolved.
    Resolved(ResolvedLink),
    /// Link could not be resolved.
    Unresolved(UnresolvedLink),
}

/// Resolves a Lattice ID to a relative file path from the source document.
///
/// Looks up the target document by ID in the index and computes the relative
/// path from the source document's directory to the target document's path.
///
/// # Arguments
///
/// * `conn` - Database connection to the index
/// * `source_path` - Path of the document containing the link
/// * `target_id` - The Lattice ID to resolve
///
/// # Returns
///
/// A `LinkResolution` indicating either the resolved path or the reason for
/// failure. This function does not return errors for missing targets; instead,
/// it returns `LinkResolution::Unresolved` with details about why resolution
/// failed.
///
/// # Errors
///
/// Returns `LatticeError` only for database errors, not for missing documents.
pub fn resolve(
    conn: &Connection,
    source_path: &Path,
    target_id: &LatticeId,
) -> Result<LinkResolution, LatticeError> {
    debug!(
        source = %source_path.display(),
        target_id = %target_id,
        "Resolving link"
    );

    let target_row = document_queries::lookup_by_id(conn, target_id.as_str())?;

    let Some(target) = target_row else {
        debug!(target_id = %target_id, "Target document not found");
        return Ok(LinkResolution::Unresolved(UnresolvedLink {
            target_id: target_id.clone(),
            reason: UnresolvedReason::TargetNotFound,
        }));
    };

    let target_path = Path::new(&target.path);
    let relative_path = compute_relative_path(source_path, target_path);

    let Some(relative_str) = relative_path.to_str() else {
        debug!(
            target_id = %target_id,
            path = %target_path.display(),
            "Target path contains invalid UTF-8"
        );
        return Ok(LinkResolution::Unresolved(UnresolvedLink {
            target_id: target_id.clone(),
            reason: UnresolvedReason::InvalidTargetPath { path: target.path },
        }));
    };

    let link_url = format!("{relative_str}#{target_id}");
    debug!(
        target_id = %target_id,
        relative_path = %relative_str,
        link_url = %link_url,
        "Link resolved successfully"
    );

    Ok(LinkResolution::Resolved(ResolvedLink {
        target_id: target_id.clone(),
        relative_path: relative_str.to_string(),
        link_url,
    }))
}

/// Resolves multiple links from a source document.
///
/// Convenience wrapper for calling `resolve` on each target ID. Useful when
/// processing multiple links from the same source document.
pub fn resolve_batch(
    conn: &Connection,
    source_path: &Path,
    target_ids: &[LatticeId],
) -> Result<Vec<LinkResolution>, LatticeError> {
    debug!(
        source = %source_path.display(),
        count = target_ids.len(),
        "Resolving batch of links"
    );

    target_ids.iter().map(|id| resolve(conn, source_path, id)).collect()
}

/// Computes a relative path from a base directory to a target path.
///
/// Uses standard filesystem conventions:
/// - `.` for same directory
/// - `..` for parent directory traversal
pub fn relative_path_between(base: &Path, target: &Path) -> PathBuf {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let common_prefix_len =
        base_components.iter().zip(target_components.iter()).take_while(|(a, b)| a == b).count();

    let ups_needed = base_components.len() - common_prefix_len;
    let mut result = PathBuf::new();

    for _ in 0..ups_needed {
        result.push("..");
    }

    for component in target_components.iter().skip(common_prefix_len) {
        result.push(component);
    }

    if result.as_os_str().is_empty() { PathBuf::from(".") } else { result }
}

/// Computes the relative path from a source document to a target document.
///
/// The relative path is computed from the source document's parent directory
/// to the target document's path, using `.` and `..` as needed.
fn compute_relative_path(source_path: &Path, target_path: &Path) -> PathBuf {
    let source_dir = source_path.parent().unwrap_or(Path::new(""));
    relative_path_between(source_dir, target_path)
}
