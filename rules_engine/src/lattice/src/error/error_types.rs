use std::path::PathBuf;

use crate::error::exit_codes;

/// All recoverable errors in Lattice.
///
/// Errors are categorized based on ownership: who is responsible for fixing
/// the problem? User errors (categories 2-4) indicate problems the user can
/// fix. System errors (category 1) are handled via panic, not this enum.
#[derive(Debug, thiserror::Error)]
pub enum LatticeError {
    // ========================================================================
    // Validation Errors (Exit Code 2)
    // Invalid document content that the user can fix.
    // ========================================================================
    #[error("Document {id} has invalid frontmatter: {reason}")]
    InvalidFrontmatter { id: String, path: PathBuf, reason: String },

    #[error("Invalid Lattice ID format: {value}")]
    MalformedId { value: String },

    #[error("Duplicate Lattice ID {id} found in {path1} and {path2}")]
    DuplicateId { id: String, path1: PathBuf, path2: PathBuf },

    #[error("Reference to unknown ID {target} in {source_id} at line {line}")]
    BrokenReference { source_id: String, target: String, path: PathBuf, line: usize },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String, involved_ids: Vec<String> },

    #[error("Missing required field '{field}' in {path}")]
    MissingRequiredField { field: String, path: PathBuf },

    #[error("Invalid field value for '{field}' in {path}: {reason}")]
    InvalidFieldValue { field: String, path: PathBuf, reason: String },

    #[error("Document {path} exceeds maximum size: {lines} lines (limit: {limit})")]
    DocumentTooLarge { path: PathBuf, lines: usize, limit: usize },

    #[error("Invalid document structure in {path}: {reason}")]
    InvalidStructure { path: PathBuf, reason: String },

    #[error("YAML parsing failed in {path}: {reason}")]
    YamlParseError { path: PathBuf, reason: String },

    #[error("Failed to parse configuration file {path}: {reason}")]
    ConfigParseError { path: PathBuf, reason: String },

    #[error("Invalid configuration value for '{field}': {reason}")]
    ConfigValidationError { field: String, reason: String },

    // ========================================================================
    // User Input Errors (Exit Code 3)
    // Invalid command-line arguments or user input.
    // ========================================================================
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("Unknown option: {option}")]
    UnknownOption { option: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf, reason: String },

    #[error("Path is not within repository: {path}")]
    PathOutsideRepository { path: PathBuf },

    #[error("Conflicting options: {option1} and {option2} cannot be used together")]
    ConflictingOptions { option1: String, option2: String },

    #[error("Missing required argument: {argument}")]
    MissingArgument { argument: String },

    #[error("Invalid ID format in argument: {value}")]
    InvalidIdArgument { value: String },

    #[error("Cannot perform operation: {reason}")]
    OperationNotAllowed { reason: String },

    #[error("Path already exists: {path}")]
    PathAlreadyExists { path: PathBuf },

    // ========================================================================
    // Not Found Errors (Exit Code 4)
    // Requested resources do not exist.
    // ========================================================================
    #[error("Document {id} not found")]
    DocumentNotFound { id: String },

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("No documents matching filter")]
    NoResults { filter_description: String },

    #[error("No root document found in directory: {path}")]
    RootDocumentNotFound { path: PathBuf },

    #[error("Parent document {id} not found")]
    ParentNotFound { id: String },

    #[error("Claim not found for document {id}")]
    ClaimNotFound { id: String },

    #[error("Label not found: {label}")]
    LabelNotFound { label: String },

    // ========================================================================
    // I/O Errors (various exit codes depending on context)
    // File system and external system errors.
    // ========================================================================
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("Failed to read file {path}: {reason}")]
    ReadError { path: PathBuf, reason: String },

    #[error("Failed to write file {path}: {reason}")]
    WriteError { path: PathBuf, reason: String },

    #[error("Git operation failed: {operation}: {reason}")]
    GitError { operation: String, reason: String },

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    #[error("Index is corrupted: {reason}")]
    IndexCorrupted { reason: String },
}

/// Error categories for structured output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Validation,
    UserInput,
    NotFound,
}

impl LatticeError {
    /// Returns the exit code for this error category.
    pub fn exit_code(&self) -> u8 {
        match self {
            LatticeError::InvalidFrontmatter { .. }
            | LatticeError::MalformedId { .. }
            | LatticeError::DuplicateId { .. }
            | LatticeError::BrokenReference { .. }
            | LatticeError::CircularDependency { .. }
            | LatticeError::MissingRequiredField { .. }
            | LatticeError::InvalidFieldValue { .. }
            | LatticeError::DocumentTooLarge { .. }
            | LatticeError::InvalidStructure { .. }
            | LatticeError::YamlParseError { .. }
            | LatticeError::ConfigParseError { .. }
            | LatticeError::ConfigValidationError { .. } => exit_codes::VALIDATION_ERROR,

            LatticeError::InvalidArgument { .. }
            | LatticeError::UnknownOption { .. }
            | LatticeError::InvalidPath { .. }
            | LatticeError::PathOutsideRepository { .. }
            | LatticeError::ConflictingOptions { .. }
            | LatticeError::MissingArgument { .. }
            | LatticeError::InvalidIdArgument { .. }
            | LatticeError::OperationNotAllowed { .. }
            | LatticeError::PathAlreadyExists { .. } => exit_codes::USER_INPUT_ERROR,

            LatticeError::DocumentNotFound { .. }
            | LatticeError::FileNotFound { .. }
            | LatticeError::NoResults { .. }
            | LatticeError::RootDocumentNotFound { .. }
            | LatticeError::ParentNotFound { .. }
            | LatticeError::ClaimNotFound { .. }
            | LatticeError::LabelNotFound { .. } => exit_codes::NOT_FOUND,

            LatticeError::PermissionDenied { .. }
            | LatticeError::ReadError { .. }
            | LatticeError::WriteError { .. }
            | LatticeError::GitError { .. }
            | LatticeError::DatabaseError { .. }
            | LatticeError::IndexCorrupted { .. } => exit_codes::USER_INPUT_ERROR,
        }
    }

    /// Returns the error code string (e.g., "E001", "E002").
    pub fn error_code(&self) -> &'static str {
        match self {
            LatticeError::InvalidFrontmatter { .. } => "E001",
            LatticeError::MalformedId { .. } => "E002",
            LatticeError::DuplicateId { .. } => "E003",
            LatticeError::BrokenReference { .. } => "E004",
            LatticeError::CircularDependency { .. } => "E005",
            LatticeError::MissingRequiredField { .. } => "E006",
            LatticeError::InvalidFieldValue { .. } => "E007",
            LatticeError::DocumentTooLarge { .. } => "E008",
            LatticeError::InvalidStructure { .. } => "E009",
            LatticeError::YamlParseError { .. } => "E010",
            LatticeError::ConfigParseError { .. } => "E011",
            LatticeError::ConfigValidationError { .. } => "E012",
            LatticeError::InvalidArgument { .. } => "E013",
            LatticeError::UnknownOption { .. } => "E014",
            LatticeError::InvalidPath { .. } => "E015",
            LatticeError::PathOutsideRepository { .. } => "E016",
            LatticeError::ConflictingOptions { .. } => "E017",
            LatticeError::MissingArgument { .. } => "E018",
            LatticeError::InvalidIdArgument { .. } => "E019",
            LatticeError::OperationNotAllowed { .. } => "E020",
            LatticeError::PathAlreadyExists { .. } => "E034",
            LatticeError::DocumentNotFound { .. } => "E021",
            LatticeError::FileNotFound { .. } => "E022",
            LatticeError::NoResults { .. } => "E023",
            LatticeError::RootDocumentNotFound { .. } => "E024",
            LatticeError::ParentNotFound { .. } => "E025",
            LatticeError::ClaimNotFound { .. } => "E026",
            LatticeError::LabelNotFound { .. } => "E027",
            LatticeError::PermissionDenied { .. } => "E028",
            LatticeError::ReadError { .. } => "E029",
            LatticeError::WriteError { .. } => "E030",
            LatticeError::GitError { .. } => "E031",
            LatticeError::DatabaseError { .. } => "E032",
            LatticeError::IndexCorrupted { .. } => "E033",
        }
    }

    /// Returns the error category.
    pub fn category(&self) -> ErrorCategory {
        match self.exit_code() {
            exit_codes::VALIDATION_ERROR => ErrorCategory::Validation,
            exit_codes::USER_INPUT_ERROR => ErrorCategory::UserInput,
            exit_codes::NOT_FOUND => ErrorCategory::NotFound,
            _ => ErrorCategory::UserInput,
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "validation"),
            ErrorCategory::UserInput => write!(f, "user_input"),
            ErrorCategory::NotFound => write!(f, "not_found"),
        }
    }
}
