use std::path::{Path, PathBuf};

use tracing::debug;

use crate::task::{closed_directory, root_detection};

/// Path segment that indicates a document is in a tasks directory.
const TASKS_DIR_SEGMENT: &str = "/tasks/";
/// Path segment that indicates a document is in a docs directory.
const DOCS_DIR_SEGMENT: &str = "/docs/";
/// Directory name for tasks.
const TASKS_DIR_NAME: &str = "tasks";
/// Directory name for documentation.
const DOCS_DIR_NAME: &str = "docs";

/// Warnings that can be issued for document placement.
///
/// These correspond to linter warnings W017-W019 as defined in the Lattice
/// linter appendix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationWarning {
    /// W017: Document is not in a standard location (root, tasks/, or docs/).
    NotInStandardLocation,

    /// W018: Task document (has `task-type`) is in a docs/ directory.
    TaskInDocsDir,

    /// W019: Knowledge base document (no `task-type`) is in a tasks/ directory.
    KnowledgeBaseInTasksDir,
}

/// Checks if a path is under a `tasks/` directory.
///
/// This is a fast path check that examines the path string directly without
/// filesystem access. The path can be at any depth within the tasks directory.
///
/// Note: Documents in `.closed/` subdirectories under `tasks/` are also
/// considered to be in the tasks directory.
pub fn is_in_tasks_dir(path: &str) -> bool {
    path.contains(TASKS_DIR_SEGMENT)
}

/// Checks if a path is under a `docs/` directory.
///
/// This is a fast path check that examines the path string directly without
/// filesystem access. The path can be at any depth within the docs directory.
pub fn is_in_docs_dir(path: &str) -> bool {
    path.contains(DOCS_DIR_SEGMENT)
}

/// Validates that a document is in an appropriate location.
///
/// Documents must be in one of these locations:
/// - Root document (filename matches directory name)
/// - `tasks/` subdirectory (for task documents with `task-type`)
/// - `docs/` subdirectory (for knowledge base documents without `task-type`)
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document
/// * `has_task_type` - Whether the document has a `task-type` field
///
/// # Returns
///
/// `None` if the location is valid, or `Some(LocationWarning)` indicating
/// the type of placement issue.
pub fn validate_location(doc_path: &Path, has_task_type: bool) -> Option<LocationWarning> {
    let path_str = doc_path.to_string_lossy();

    if root_detection::is_root_document(doc_path) {
        debug!(
            path = %doc_path.display(),
            "Document is a root document, location is valid"
        );
        return None;
    }

    let in_tasks = is_in_tasks_dir(&path_str);
    let in_docs = is_in_docs_dir(&path_str);

    if !in_tasks && !in_docs {
        debug!(
            path = %doc_path.display(),
            "Document is not in tasks/ or docs/ directory"
        );
        return Some(LocationWarning::NotInStandardLocation);
    }

    if has_task_type && in_docs && !in_tasks {
        debug!(
            path = %doc_path.display(),
            "Task document found in docs/ directory"
        );
        return Some(LocationWarning::TaskInDocsDir);
    }

    if !has_task_type && in_tasks && !closed_directory::is_in_closed(&path_str) {
        debug!(
            path = %doc_path.display(),
            "Knowledge base document found in tasks/ directory"
        );
        return Some(LocationWarning::KnowledgeBaseInTasksDir);
    }

    debug!(
        path = %doc_path.display(),
        has_task_type = has_task_type,
        in_tasks = in_tasks,
        in_docs = in_docs,
        "Document location is valid"
    );

    None
}

/// Suggests the expected location for a document.
///
/// Given a document's current path and whether it has a task-type, returns
/// the path where it should be located according to Lattice conventions.
///
/// For root documents, returns the current path unchanged.
///
/// For non-root documents:
/// - Task documents should be in `{parent}/tasks/`
/// - Knowledge base documents should be in `{parent}/docs/`
///
/// # Arguments
///
/// * `doc_path` - Relative path to the document
/// * `has_task_type` - Whether the document has a `task-type` field
///
/// # Returns
///
/// The expected path for the document. If the document is already in the
/// correct location, this will match the input path.
pub fn expected_location(doc_path: &Path, has_task_type: bool) -> PathBuf {
    if root_detection::is_root_document(doc_path) {
        return doc_path.to_path_buf();
    }

    let path_str = doc_path.to_string_lossy();
    let in_tasks = is_in_tasks_dir(&path_str);
    let in_docs = is_in_docs_dir(&path_str);
    let in_closed = closed_directory::is_in_closed(&path_str);

    if has_task_type {
        if in_tasks {
            return doc_path.to_path_buf();
        }
        return relocate_to_dir(doc_path, TASKS_DIR_NAME, in_docs);
    }

    if in_docs {
        return doc_path.to_path_buf();
    }

    if in_tasks && !in_closed {
        return relocate_to_dir(doc_path, DOCS_DIR_NAME, true);
    }

    relocate_to_dir(doc_path, DOCS_DIR_NAME, false)
}

/// Returns the target directory name for a document type.
///
/// # Arguments
///
/// * `has_task_type` - Whether the document has a `task-type` field
///
/// # Returns
///
/// "tasks" for task documents, "docs" for knowledge base documents.
pub fn target_dir_name(has_task_type: bool) -> &'static str {
    if has_task_type { TASKS_DIR_NAME } else { DOCS_DIR_NAME }
}

impl LocationWarning {
    /// Returns the warning code string (e.g., "W017").
    pub fn code(&self) -> &'static str {
        match self {
            LocationWarning::NotInStandardLocation => "W017",
            LocationWarning::TaskInDocsDir => "W018",
            LocationWarning::KnowledgeBaseInTasksDir => "W019",
        }
    }

    /// Returns a human-readable message for this warning.
    pub fn message(&self, path: &Path) -> String {
        match self {
            LocationWarning::NotInStandardLocation => {
                format!("{} is not in tasks/ or docs/ directory", path.display())
            }
            LocationWarning::TaskInDocsDir => {
                format!("{} is a task but located in docs/", path.display())
            }
            LocationWarning::KnowledgeBaseInTasksDir => {
                format!("{} is a knowledge base document but located in tasks/", path.display())
            }
        }
    }
}

/// Relocates a document path to a target directory.
///
/// If the document is in a source directory (`tasks/` or `docs/`), replaces
/// that directory with the target. Otherwise, inserts the target directory
/// before the filename.
fn relocate_to_dir(doc_path: &Path, target_dir: &str, in_source_dir: bool) -> PathBuf {
    let file_name = doc_path
        .file_name()
        .unwrap_or_else(|| panic!("Document path has no filename: {}", doc_path.display()));

    if in_source_dir {
        let path_str = doc_path.to_string_lossy();

        let (replacement_segment, search_segment) = if is_in_tasks_dir(&path_str) {
            (format!("/{target_dir}/"), TASKS_DIR_SEGMENT)
        } else {
            (format!("/{target_dir}/"), DOCS_DIR_SEGMENT)
        };

        let new_path_str = path_str.replacen(search_segment, &replacement_segment, 1);
        return PathBuf::from(new_path_str);
    }

    let parent = doc_path.parent().unwrap_or_else(|| Path::new(""));
    parent.join(target_dir).join(file_name)
}
