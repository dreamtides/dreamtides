use std::path::PathBuf;

use crate::error::error_types::LatticeError;

/// Returns a user-friendly suggestion for how to fix this error.
pub fn suggestion(error: &LatticeError) -> Option<String> {
    match error {
        LatticeError::InvalidFrontmatter { .. } => {
            Some("Check the YAML frontmatter syntax and required fields".to_string())
        }
        LatticeError::MalformedId { value } => Some(format!(
            "Lattice IDs must start with 'L' followed by Base32 characters (A-Z, 2-7). \
             Got: {value}"
        )),
        LatticeError::DuplicateId { id, .. } => Some(format!(
            "Run 'lat track --force {id}' to regenerate IDs for documents with duplicates"
        )),
        LatticeError::BrokenReference { target, .. } => Some(format!(
            "Create the target document with 'lat create' or correct the ID '{target}'"
        )),
        LatticeError::CircularDependency { .. } => {
            Some("Remove one of the dependency links to break the cycle".to_string())
        }
        LatticeError::MissingRequiredField { field, path } => Some(format!(
            "Add the '{field}' field to the frontmatter in {path}",
            path = path.display()
        )),
        LatticeError::InvalidFieldValue { field, .. } => {
            Some(format!("Check the valid values for the '{field}' field"))
        }
        LatticeError::DocumentTooLarge { path, .. } => Some(format!(
            "Run 'lat split {path}' to divide the document into smaller files",
            path = path.display()
        )),
        LatticeError::InvalidStructure { .. } => {
            Some("Check the document path structure".to_string())
        }
        LatticeError::YamlParseError { .. } => {
            Some("Check YAML syntax: proper indentation, colons, and quotes".to_string())
        }
        LatticeError::ConfigParseError { path, .. } => {
            Some(format!("Check TOML syntax in {path}", path = path.display()))
        }
        LatticeError::ConfigValidationError { field, .. } => {
            Some(format!("Check the valid values for the '{field}' configuration field"))
        }
        LatticeError::DocumentNotFound { id } => {
            Some(format!("Verify the ID '{id}' is correct or run 'lat list' to find documents"))
        }
        LatticeError::FileNotFound { path } => {
            Some(format!("Verify the path exists: {path}", path = path.display()))
        }
        LatticeError::NoResults { filter_description } => {
            Some(format!("Try broader filters. Current filter: {filter_description}"))
        }
        LatticeError::RootDocumentNotFound { path } => Some(format!(
            "Create a root document with filename matching the directory: {dir}/{dir}.md",
            dir = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()
        )),
        LatticeError::ParentNotFound { id } => {
            Some(format!("Create the parent document '{id}' first"))
        }
        LatticeError::ClaimNotFound { id } => {
            Some(format!("Run 'lat claim {id}' to claim this document"))
        }
        LatticeError::LabelNotFound { label } => {
            Some(format!("Run 'lat label add <id> {label}' to create the label"))
        }
        LatticeError::DependencyNotFound { source_id, target_id } => {
            Some(format!("Run 'lat dep add {source_id} {target_id}' to create the dependency"))
        }
        LatticeError::PermissionDenied { .. } => {
            Some("Check file permissions and try again".to_string())
        }
        LatticeError::ReadError { .. } | LatticeError::WriteError { .. } => {
            Some("Check file permissions and disk space".to_string())
        }
        LatticeError::GitError { .. } => {
            Some("Check git repository state with 'git status'".to_string())
        }
        LatticeError::DatabaseError { .. } | LatticeError::IndexCorrupted { .. } => {
            Some("Try removing .lattice/index.sqlite to rebuild the index".to_string())
        }
        LatticeError::InvalidArgument { .. }
        | LatticeError::UnknownOption { .. }
        | LatticeError::InvalidPath { .. }
        | LatticeError::PathOutsideRepository { .. }
        | LatticeError::ConflictingOptions { .. }
        | LatticeError::MissingArgument { .. }
        | LatticeError::InvalidIdArgument { .. }
        | LatticeError::OperationNotAllowed { .. } => {
            Some("Run 'lat --help' for usage".to_string())
        }
        LatticeError::PathAlreadyExists { path } => {
            Some(format!("Remove or rename the existing file at {path}", path = path.display()))
        }
        LatticeError::FmtCheckFailed { .. } => {
            Some("Run 'lat fmt' to format the files".to_string())
        }
        LatticeError::FmtErrors { .. } => {
            Some("Check the error messages above and fix the issues".to_string())
        }
        LatticeError::NoReadyTasks => {
            Some("Try 'lat ready --include-backlog' or use broader filters".to_string())
        }
        LatticeError::ClaimLimitExceeded { .. } => Some(
            "Release existing claims with 'lat claim --release <id>' or increase --max-claims"
                .to_string(),
        ),
    }
}

/// Returns the affected document IDs for this error, if any.
pub fn affected_documents(error: &LatticeError) -> Vec<String> {
    match error {
        LatticeError::InvalidFrontmatter { id, .. } => vec![id.clone()],
        LatticeError::DuplicateId { id, .. } => vec![id.clone()],
        LatticeError::BrokenReference { source_id, target, .. } => {
            vec![source_id.clone(), target.clone()]
        }
        LatticeError::CircularDependency { involved_ids, .. } => involved_ids.clone(),
        LatticeError::DocumentNotFound { id } => vec![id.clone()],
        LatticeError::ParentNotFound { id } => vec![id.clone()],
        LatticeError::ClaimNotFound { id } => vec![id.clone()],
        LatticeError::DependencyNotFound { source_id, target_id } => {
            vec![source_id.clone(), target_id.clone()]
        }
        _ => vec![],
    }
}

/// Returns the file path associated with this error, if any.
pub fn error_path(error: &LatticeError) -> Option<&PathBuf> {
    match error {
        LatticeError::InvalidFrontmatter { path, .. }
        | LatticeError::BrokenReference { path, .. }
        | LatticeError::MissingRequiredField { path, .. }
        | LatticeError::InvalidFieldValue { path, .. }
        | LatticeError::DocumentTooLarge { path, .. }
        | LatticeError::InvalidStructure { path, .. }
        | LatticeError::YamlParseError { path, .. }
        | LatticeError::ConfigParseError { path, .. }
        | LatticeError::InvalidPath { path, .. }
        | LatticeError::PathOutsideRepository { path }
        | LatticeError::FileNotFound { path }
        | LatticeError::RootDocumentNotFound { path }
        | LatticeError::PermissionDenied { path }
        | LatticeError::ReadError { path, .. }
        | LatticeError::WriteError { path, .. }
        | LatticeError::PathAlreadyExists { path } => Some(path),

        LatticeError::DuplicateId { path1, .. } => Some(path1),

        LatticeError::MalformedId { .. }
        | LatticeError::CircularDependency { .. }
        | LatticeError::ConfigValidationError { .. }
        | LatticeError::InvalidArgument { .. }
        | LatticeError::UnknownOption { .. }
        | LatticeError::ConflictingOptions { .. }
        | LatticeError::MissingArgument { .. }
        | LatticeError::InvalidIdArgument { .. }
        | LatticeError::OperationNotAllowed { .. }
        | LatticeError::DocumentNotFound { .. }
        | LatticeError::NoResults { .. }
        | LatticeError::ParentNotFound { .. }
        | LatticeError::ClaimNotFound { .. }
        | LatticeError::LabelNotFound { .. }
        | LatticeError::DependencyNotFound { .. }
        | LatticeError::NoReadyTasks
        | LatticeError::ClaimLimitExceeded { .. }
        | LatticeError::GitError { .. }
        | LatticeError::DatabaseError { .. }
        | LatticeError::IndexCorrupted { .. }
        | LatticeError::FmtCheckFailed { .. }
        | LatticeError::FmtErrors { .. } => None,
    }
}

/// Returns the line number where the error occurred, if applicable.
pub fn error_line(error: &LatticeError) -> Option<usize> {
    match error {
        LatticeError::BrokenReference { line, .. } => Some(*line),
        _ => None,
    }
}

/// Formats an error for terminal display with color hints.
///
/// Returns a tuple of (prefix, message, suggestion) for styling.
pub fn format_for_terminal(error: &LatticeError) -> (String, String, Option<String>) {
    let prefix = format!("[{}]", error.error_code());
    let message = error.to_string();
    let hint = suggestion(error);
    (prefix, message, hint)
}
