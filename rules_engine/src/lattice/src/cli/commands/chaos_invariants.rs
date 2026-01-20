use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::Connection;
use tracing::{debug, warn};

use crate::document::document_reader;
use crate::id::lattice_id::LatticeId;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;
use crate::link::link_extractor::{self, LinkCategory};

/// Details of an invariant violation detected by chaos monkey.
#[derive(Debug, Clone)]
pub struct InvariantViolation {
    /// Which invariant was violated.
    pub invariant: InvariantKind,
    /// Human-readable description of the violation.
    pub description: String,
    /// Relevant file paths, if any.
    pub affected_paths: Vec<PathBuf>,
    /// Relevant IDs, if any.
    pub affected_ids: Vec<String>,
}

/// Types of invariants checked by chaos monkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantKind {
    /// Index contains ID not in filesystem.
    IndexHasOrphanedId,
    /// Filesystem has document not in index.
    FilesystemHasUnindexedDocument,
    /// Two files share the same Lattice ID.
    DuplicateId,
    /// ID in index doesn't match Lattice ID format.
    MalformedIdInIndex,
    /// A panic occurred during operation.
    Panic,
    /// Index is_closed doesn't match path.
    ClosedStateInconsistency,
    /// Index is_root doesn't match filename.
    RootStateInconsistency,
    /// Git operation failed unexpectedly.
    GitOperationFailed,
    /// Link path in document doesn't match current file location.
    LinkPathMismatch,
}

/// Checks all invariants after an operation.
///
/// See appendix_chaos_monkey.md for the full list of invariants.
pub fn check_all(conn: &Connection, repo_root: &Path) -> Result<(), InvariantViolation> {
    check_index_filesystem_consistency(conn, repo_root)?;
    check_id_uniqueness(conn)?;
    check_id_format(conn)?;
    check_closed_state_consistency(conn)?;
    check_root_state_consistency(conn)?;
    check_git_state_validity(repo_root)?;
    check_link_path_validity(conn, repo_root)?;
    Ok(())
}

/// Finds all markdown files in a directory recursively.
pub fn find_markdown_files(dir: &Path) -> Result<Vec<PathBuf>, InvariantViolation> {
    let mut files = Vec::new();
    find_markdown_files_recursive(dir, &mut files)?;
    Ok(files)
}

impl InvariantKind {
    /// Human-readable name for this invariant.
    pub fn name(&self) -> &'static str {
        match self {
            InvariantKind::IndexHasOrphanedId => "index-orphaned-id",
            InvariantKind::FilesystemHasUnindexedDocument => "filesystem-unindexed",
            InvariantKind::DuplicateId => "duplicate-id",
            InvariantKind::MalformedIdInIndex => "malformed-id",
            InvariantKind::Panic => "panic",
            InvariantKind::ClosedStateInconsistency => "closed-state-mismatch",
            InvariantKind::RootStateInconsistency => "root-state-mismatch",
            InvariantKind::GitOperationFailed => "git-operation-failed",
            InvariantKind::LinkPathMismatch => "link-path-mismatch",
        }
    }
}

/// Creates a filter that matches all documents including closed ones.
fn all_documents_filter() -> DocumentFilter {
    DocumentFilter { include_closed: true, ..Default::default() }
}

/// Queries all documents from the index for an invariant check.
fn query_all_docs(
    conn: &Connection,
    invariant: InvariantKind,
) -> Result<Vec<DocumentRow>, InvariantViolation> {
    document_queries::query(conn, &all_documents_filter()).map_err(|e| InvariantViolation {
        invariant,
        description: format!("Failed to query index: {e}"),
        affected_paths: vec![],
        affected_ids: vec![],
    })
}

/// Checks that every ID in the index has a corresponding file (invariant 1).
fn check_index_filesystem_consistency(
    conn: &Connection,
    repo_root: &Path,
) -> Result<(), InvariantViolation> {
    let index_docs = query_all_docs(conn, InvariantKind::IndexHasOrphanedId)?;

    for doc in &index_docs {
        let file_path = repo_root.join(&doc.path);
        if !file_path.exists() {
            return Err(InvariantViolation {
                invariant: InvariantKind::IndexHasOrphanedId,
                description: format!(
                    "Index contains ID {} at path {} but file does not exist",
                    doc.id, doc.path
                ),
                affected_paths: vec![file_path],
                affected_ids: vec![doc.id.clone()],
            });
        }
    }

    let filesystem_docs = find_markdown_files(repo_root)?;
    let indexed_paths: HashSet<_> = index_docs.iter().map(|d| PathBuf::from(&d.path)).collect();

    for file_path in &filesystem_docs {
        if let Ok(relative) = file_path.strip_prefix(repo_root) {
            let relative_str = relative.to_string_lossy().to_string();
            if relative_str.contains(".lattice") || relative_str.contains(".git") {
                continue;
            }

            let doc_result = document_reader::read(file_path);
            if let Ok(doc) = doc_result
                && !indexed_paths.contains(relative)
            {
                return Err(InvariantViolation {
                    invariant: InvariantKind::FilesystemHasUnindexedDocument,
                    description: format!(
                        "Filesystem has document at {} with ID {} but it's not in index",
                        relative.display(),
                        doc.frontmatter.lattice_id
                    ),
                    affected_paths: vec![file_path.clone()],
                    affected_ids: vec![doc.frontmatter.lattice_id.to_string()],
                });
            }
        }
    }

    Ok(())
}

/// Checks that no two files share the same ID (invariant 2).
fn check_id_uniqueness(conn: &Connection) -> Result<(), InvariantViolation> {
    let docs = query_all_docs(conn, InvariantKind::DuplicateId)?;

    let mut id_to_path: HashMap<String, String> = HashMap::new();
    for doc in docs {
        if let Some(existing_path) = id_to_path.get(&doc.id) {
            return Err(InvariantViolation {
                invariant: InvariantKind::DuplicateId,
                description: format!(
                    "Duplicate ID {} found in {} and {}",
                    doc.id, existing_path, doc.path
                ),
                affected_paths: vec![PathBuf::from(existing_path), PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
        id_to_path.insert(doc.id.clone(), doc.path.clone());
    }

    Ok(())
}

/// Checks that all IDs in the index are valid Lattice IDs (invariant 3).
fn check_id_format(conn: &Connection) -> Result<(), InvariantViolation> {
    let docs = query_all_docs(conn, InvariantKind::MalformedIdInIndex)?;

    for doc in docs {
        if doc.id.parse::<LatticeId>().is_err() {
            return Err(InvariantViolation {
                invariant: InvariantKind::MalformedIdInIndex,
                description: format!(
                    "Index contains malformed ID '{}' at path {}",
                    doc.id, doc.path
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that is_closed in index matches path containing .closed/ (invariant
/// 6).
fn check_closed_state_consistency(conn: &Connection) -> Result<(), InvariantViolation> {
    let docs = query_all_docs(conn, InvariantKind::ClosedStateInconsistency)?;

    for doc in docs {
        let path_indicates_closed =
            doc.path.contains("/tasks/.closed/") || doc.path.contains("/.closed/");

        if doc.is_closed != path_indicates_closed {
            return Err(InvariantViolation {
                invariant: InvariantKind::ClosedStateInconsistency,
                description: format!(
                    "Document {} has is_closed={} but path '{}' {} .closed/",
                    doc.id,
                    doc.is_closed,
                    doc.path,
                    if path_indicates_closed { "contains" } else { "does not contain" }
                ),
                affected_paths: vec![PathBuf::from(&doc.path)],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that is_root in index matches filename = directory name (invariant
/// 7).
fn check_root_state_consistency(conn: &Connection) -> Result<(), InvariantViolation> {
    let docs = query_all_docs(conn, InvariantKind::RootStateInconsistency)?;

    for doc in docs {
        let path = PathBuf::from(&doc.path);
        let is_root_by_path = is_root_document(&path);

        if doc.is_root != is_root_by_path {
            return Err(InvariantViolation {
                invariant: InvariantKind::RootStateInconsistency,
                description: format!(
                    "Document {} has is_root={} but path '{}' {} a root document",
                    doc.id,
                    doc.is_root,
                    doc.path,
                    if is_root_by_path { "is" } else { "is not" }
                ),
                affected_paths: vec![path],
                affected_ids: vec![doc.id],
            });
        }
    }

    Ok(())
}

/// Checks that git state is valid after lat operations (invariant 4).
///
/// This verifies that lat operations don't leave the repository in an
/// unexpectedly dirty state. The check runs `git status --porcelain` and
/// verifies the output is parseable. Note: We don't require a clean state
/// since lat operations may create uncommitted changes; we just verify git
/// is functional.
fn check_git_state_validity(repo_root: &Path) -> Result<(), InvariantViolation> {
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| InvariantViolation {
            invariant: InvariantKind::GitOperationFailed,
            description: format!("Failed to run git status: {e}"),
            affected_paths: vec![repo_root.to_path_buf()],
            affected_ids: vec![],
        })?;

    if !status_output.status.success() {
        return Err(InvariantViolation {
            invariant: InvariantKind::GitOperationFailed,
            description: format!(
                "git status failed: {}",
                String::from_utf8_lossy(&status_output.stderr)
            ),
            affected_paths: vec![repo_root.to_path_buf()],
            affected_ids: vec![],
        });
    }

    let rev_parse_output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| InvariantViolation {
            invariant: InvariantKind::GitOperationFailed,
            description: format!("Failed to run git rev-parse: {e}"),
            affected_paths: vec![repo_root.to_path_buf()],
            affected_ids: vec![],
        })?;

    if !rev_parse_output.status.success() {
        return Err(InvariantViolation {
            invariant: InvariantKind::GitOperationFailed,
            description: format!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&rev_parse_output.stderr)
            ),
            affected_paths: vec![repo_root.to_path_buf()],
            affected_ids: vec![],
        });
    }

    debug!("Git state validity check passed");
    Ok(())
}

/// Checks that link paths in documents match current file locations (invariant
/// 9).
///
/// After lat close/lat reopen, links with Lattice ID fragments should be
/// rewritten to point to the document's current path. This check verifies that
/// links with valid Lattice IDs point to the correct location.
///
/// Note: Broken links to deleted documents are NOT invariant violations per
/// the spec - only links where the target document exists but at a different
/// path are violations.
fn check_link_path_validity(conn: &Connection, repo_root: &Path) -> Result<(), InvariantViolation> {
    let docs = query_all_docs(conn, InvariantKind::LinkPathMismatch)?;
    let id_to_path: HashMap<String, String> =
        docs.iter().map(|d| (d.id.clone(), d.path.clone())).collect();

    for doc in &docs {
        let file_path = repo_root.join(&doc.path);
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                warn!(path = %file_path.display(), error = %e, "Failed to read file for link check");
                continue;
            }
        };

        let extraction = link_extractor::extract(&content);

        for link in extraction.links {
            if link.link_type == LinkCategory::External || link.link_type == LinkCategory::Other {
                continue;
            }

            // Only check links that have both a path and a Lattice ID fragment.
            // Links without an ID cannot be verified (we don't know what they
            // reference). Broken links to deleted documents (ID not in index)
            // are allowed per the spec.
            if let (Some(link_path), Some(fragment)) = (&link.path, &link.fragment) {
                let fragment_str = fragment.to_string();

                // Look up the target document by its Lattice ID
                if let Some(current_path) = id_to_path.get(&fragment_str) {
                    // Document exists - verify the link path matches
                    let source_dir = Path::new(&doc.path).parent().unwrap_or_else(|| Path::new(""));
                    let resolved = source_dir.join(link_path);
                    let normalized = normalize_path(&resolved);
                    let normalized_str = normalized.to_string_lossy().to_string();

                    if normalized_str != *current_path {
                        return Err(InvariantViolation {
                            invariant: InvariantKind::LinkPathMismatch,
                            description: format!(
                                "Document {} has link to '{}' (ID {}) but document is now at '{}'",
                                doc.id, link_path, fragment_str, current_path
                            ),
                            affected_paths: vec![file_path.clone(), PathBuf::from(current_path)],
                            affected_ids: vec![doc.id.clone(), fragment_str],
                        });
                    }
                }
                // If the ID doesn't exist in index, the document was deleted.
                // Broken links to deleted documents are NOT violations.
            }
        }
    }

    debug!("Link path validity check passed");
    Ok(())
}

/// Normalizes a path by resolving . and .. components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

/// Checks if a path represents a root document.
fn is_root_document(path: &Path) -> bool {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent_name =
        path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("");

    file_stem == parent_name || file_stem.starts_with("00_")
}

fn find_markdown_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), InvariantViolation> {
    let entries = fs::read_dir(dir).map_err(|e| InvariantViolation {
        invariant: InvariantKind::FilesystemHasUnindexedDocument,
        description: format!("Failed to read directory {}: {e}", dir.display()),
        affected_paths: vec![dir.to_path_buf()],
        affected_ids: vec![],
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| InvariantViolation {
            invariant: InvariantKind::FilesystemHasUnindexedDocument,
            description: format!("Failed to read directory entry: {e}"),
            affected_paths: vec![dir.to_path_buf()],
            affected_ids: vec![],
        })?;

        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != ".git" && name != ".lattice" {
                find_markdown_files_recursive(&path, files)?;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }

    Ok(())
}
