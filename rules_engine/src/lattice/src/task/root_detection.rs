use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::document::document_reader;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;

/// Constant representing the `.md` extension we expect on all documents.
const MD_EXTENSION: &str = "md";

/// Checks if a path represents a root document.
///
/// A root document has a filename (without `.md` extension) that matches its
/// containing directory name, optionally prefixed with an underscore. For
/// example:
/// - `api/api.md` → root (filename "api" matches directory "api")
/// - `api/_api.md` → root (underscore-prefixed form)
/// - `auth/auth.md` → root
/// - `auth/tasks/login.md` → NOT root (filename "login" ≠ directory "tasks")
///
/// This function examines the path structure only; it does not access the
/// filesystem or validate that the file exists.
///
/// # Arguments
///
/// * `path` - Relative path to the document
///
/// # Returns
///
/// `true` if the document is a root document, `false` otherwise.
pub fn is_root_document(path: &Path) -> bool {
    let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };

    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    if extension != MD_EXTENSION {
        return false;
    }

    let Some(parent) = path.parent() else {
        return false;
    };

    let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    let is_root = file_stem == parent_name || file_stem == format!("_{parent_name}");

    debug!(
        path = %path.display(),
        file_stem = file_stem,
        parent_name = parent_name,
        is_root = is_root,
        "Checked if document is root"
    );

    is_root
}

/// Finds the root document for a directory.
///
/// Given a directory path, looks for a document with the same name as the
/// directory. For example, for directory `api/`, looks for `api/api.md`.
///
/// This function returns the path that would be the root document if it
/// exists. Use this with the filesystem to check if the root actually exists.
///
/// # Arguments
///
/// * `dir_path` - Path to a directory (relative to repo root)
///
/// # Returns
///
/// The path to where the root document would be located (e.g., `api/api.md`
/// for directory `api/`). Returns `None` if the directory path is empty or
/// has no name component.
pub fn root_document_path_for(dir_path: &Path) -> Option<PathBuf> {
    let dir_name = dir_path.file_name()?.to_str()?;
    let root_filename = format!("{dir_name}.{MD_EXTENSION}");
    let root_path = dir_path.join(root_filename);

    debug!(
        dir_path = %dir_path.display(),
        root_path = %root_path.display(),
        "Computed root document path for directory"
    );

    Some(root_path)
}

/// Returns all possible root document paths for a directory.
///
/// Given a directory path, returns both the standard root document path
/// (`dir/dir.md`) and the underscore-prefixed form (`dir/_dir.md`).
///
/// # Arguments
///
/// * `dir_path` - Path to a directory (relative to repo root)
///
/// # Returns
///
/// A vector containing both possible root document paths. The standard form
/// is returned first, followed by the underscore-prefixed form.
/// Returns an empty vector if the directory path has no name component.
pub fn root_document_paths_for(dir_path: &Path) -> Vec<PathBuf> {
    let Some(dir_name) = dir_path.file_name().and_then(|s| s.to_str()) else {
        return vec![];
    };
    vec![
        dir_path.join(format!("{dir_name}.{MD_EXTENSION}")),
        dir_path.join(format!("_{dir_name}.{MD_EXTENSION}")),
    ]
}

/// Finds the root document for a given document path.
///
/// Walks up the directory tree from the document's parent directory to find
/// the nearest enclosing root document. The search stops at the first root
/// document found.
///
/// For example, given `api/tasks/fix_bug.md`, this would search:
/// 1. `api/tasks/tasks.md` or `api/tasks/_tasks.md` (not found)
/// 2. `api/api.md` or `api/_api.md` (found - return this)
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document
/// * `repo_root` - Absolute path to the repository root
///
/// # Returns
///
/// The relative path to the root document, or `None` if no root document
/// exists in any ancestor directory.
pub fn find_root_for(doc_path: &Path, repo_root: &Path) -> Option<PathBuf> {
    let mut current_dir = doc_path.parent()?;

    while !current_dir.as_os_str().is_empty() {
        for root_path in root_document_paths_for(current_dir) {
            let absolute_root = repo_root.join(&root_path);
            if absolute_root.is_file() {
                info!(
                    doc_path = %doc_path.display(),
                    root_path = %root_path.display(),
                    "Found root document for path"
                );
                return Some(root_path);
            }
        }
        current_dir = current_dir.parent()?;
    }

    debug!(
        doc_path = %doc_path.display(),
        "No root document found in any ancestor directory"
    );

    None
}

/// Computes the parent-id for a document.
///
/// Finds the root document for the document's containing directory and returns
/// its `lattice-id`. This is used by `lat fmt` and `lat create` to
/// auto-populate the `parent-id` field in document frontmatter.
///
/// A document's parent is the root document of its containing directory tree,
/// not the document itself. If the document is itself a root document, its
/// parent is the root document of the parent directory (if any).
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document
/// * `repo_root` - Absolute path to the repository root
///
/// # Returns
///
/// The `LatticeId` of the parent document, or an error if no parent exists or
/// the parent cannot be read.
///
/// # Errors
///
/// Returns `LatticeError::RootDocumentNotFound` if no root document exists.
/// Returns other errors if the root document cannot be read or parsed.
pub fn compute_parent_id(doc_path: &Path, repo_root: &Path) -> Result<LatticeId, LatticeError> {
    let is_root = is_root_document(doc_path);

    let search_start =
        if is_root { doc_path.parent().and_then(|p| p.parent()) } else { doc_path.parent() };

    let Some(start_dir) = search_start else {
        return Err(LatticeError::RootDocumentNotFound { path: doc_path.to_path_buf() });
    };

    let root_path = find_root_starting_from(start_dir, repo_root)
        .ok_or_else(|| LatticeError::RootDocumentNotFound { path: doc_path.to_path_buf() })?;

    let absolute_root = repo_root.join(&root_path);
    let document = document_reader::read(&absolute_root)?;
    let parent_id = document.frontmatter.lattice_id;

    info!(
        doc_path = %doc_path.display(),
        parent_id = %parent_id,
        root_path = %root_path.display(),
        "Computed parent-id for document"
    );

    Ok(parent_id)
}

/// Finds all ancestor root documents for a given document path.
///
/// Walks up the directory tree and collects all root documents encountered.
/// The returned list is ordered from the nearest ancestor to the most distant
/// (child → parent → grandparent).
///
/// This is used by the template system for composing context and acceptance
/// criteria from ancestor root documents.
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document
/// * `repo_root` - Absolute path to the repository root
///
/// # Returns
///
/// A vector of paths to ancestor root documents, ordered from nearest to
/// farthest. Returns an empty vector if no ancestor roots exist.
pub fn find_ancestors(doc_path: &Path, repo_root: &Path) -> Vec<PathBuf> {
    let mut ancestors = Vec::new();
    let is_root = is_root_document(doc_path);

    let start =
        if is_root { doc_path.parent().and_then(|p| p.parent()) } else { doc_path.parent() };

    let Some(mut current_dir) = start else {
        return ancestors;
    };

    while !current_dir.as_os_str().is_empty() {
        for root_path in root_document_paths_for(current_dir) {
            let absolute_root = repo_root.join(&root_path);
            if absolute_root.is_file() {
                ancestors.push(root_path);
                break; // Found root for this directory, move to parent
            }
        }
        if let Some(parent) = current_dir.parent() {
            current_dir = parent;
        } else {
            break;
        }
    }

    debug!(
        doc_path = %doc_path.display(),
        ancestor_count = ancestors.len(),
        "Found ancestor root documents"
    );

    ancestors
}

/// Validates that a directory hierarchy is properly structured with root
/// documents.
///
/// Checks that all non-root documents have an accessible root document in
/// their ancestor chain. This is used by `lat check` to detect orphaned
/// documents.
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document to validate
/// * `repo_root` - Absolute path to the repository root
///
/// # Returns
///
/// `Ok(())` if the document has a proper hierarchy, or an error describing
/// the hierarchy issue.
pub fn validate_hierarchy(doc_path: &Path, repo_root: &Path) -> Result<(), LatticeError> {
    if is_root_document(doc_path) {
        return Ok(());
    }

    if find_root_for(doc_path, repo_root).is_some() {
        return Ok(());
    }

    warn!(
        doc_path = %doc_path.display(),
        "Document has no root document in ancestor directories"
    );

    Err(LatticeError::RootDocumentNotFound { path: doc_path.to_path_buf() })
}

/// Finds a root document starting from a specific directory.
///
/// Similar to `find_root_for`, but starts the search from a specific directory
/// rather than from a document's parent.
fn find_root_starting_from(start_dir: &Path, repo_root: &Path) -> Option<PathBuf> {
    let mut current_dir = start_dir;

    loop {
        for root_path in root_document_paths_for(current_dir) {
            let absolute_root = repo_root.join(&root_path);
            if absolute_root.is_file() {
                debug!(
                    start_dir = %start_dir.display(),
                    root_path = %root_path.display(),
                    "Found root document starting from directory"
                );
                return Some(root_path);
            }
        }
        current_dir = current_dir.parent()?;
        if current_dir.as_os_str().is_empty() {
            break;
        }
    }

    None
}
