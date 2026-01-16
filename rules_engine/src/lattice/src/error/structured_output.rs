use std::path::PathBuf;

use serde::Serialize;

use crate::error::error_formatting;
use crate::error::error_types::{ErrorCategory, LatticeError};

/// JSON-serializable error output for `--json` mode.
#[derive(Debug, Serialize)]
pub struct StructuredError {
    pub error_code: &'static str,
    pub category: ErrorCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub affected_documents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ErrorLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_command: Option<String>,
}

/// Source location where an error occurred.
#[derive(Debug, Serialize)]
pub struct ErrorLocation {
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

/// Container for multiple errors, used when reporting validation results.
#[derive(Debug, Serialize)]
pub struct ErrorReport {
    pub errors: Vec<StructuredError>,
    pub error_count: usize,
    pub exit_code: u8,
}

impl StructuredError {
    /// Creates a structured error from a LatticeError.
    pub fn from_error(error: &LatticeError) -> Self {
        let location = error_formatting::error_path(error).map(|path| ErrorLocation {
            path: path.clone(),
            line: error_formatting::error_line(error),
            column: None,
        });

        Self {
            error_code: error.error_code(),
            category: error.category(),
            message: error.to_string(),
            affected_documents: error_formatting::affected_documents(error),
            location,
            suggestion: error_formatting::suggestion(error),
            fix_command: fix_command(error),
        }
    }

    /// Serializes this error to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"error_code":"{}","category":"{}","message":"JSON serialization failed"}}"#,
                self.error_code, self.category
            )
        })
    }

    /// Serializes this error to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            format!(
                r#"{{"error_code":"{}","category":"{}","message":"JSON serialization failed"}}"#,
                self.error_code, self.category
            )
        })
    }
}

impl ErrorReport {
    /// Creates a new error report from a list of errors.
    pub fn new(errors: Vec<&LatticeError>) -> Self {
        let structured: Vec<_> = errors.iter().map(|e| StructuredError::from_error(e)).collect();
        let error_count = structured.len();
        let exit_code = errors.first().map(|e| e.exit_code()).unwrap_or(0);
        Self { errors: structured, error_count, exit_code }
    }

    /// Serializes this report to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"errors":[],"error_count":0,"exit_code":1}"#.to_string())
    }

    /// Serializes this report to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| r#"{"errors":[],"error_count":0,"exit_code":1}"#.to_string())
    }
}

/// Returns a suggested fix command for this error, if applicable.
fn fix_command(error: &LatticeError) -> Option<String> {
    match error {
        LatticeError::DuplicateId { id, .. } => Some(format!("lat track --force {id}")),
        LatticeError::BrokenReference { target, .. } => {
            Some(format!("lat create . \"Target for {target}\""))
        }
        LatticeError::MissingRequiredField { path, .. } => {
            Some(format!("lat fmt {}", path.display()))
        }
        LatticeError::DocumentTooLarge { path, .. } => {
            Some(format!("lat split {}", path.display()))
        }
        LatticeError::ClaimNotFound { id } => Some(format!("lat claim {id}")),
        LatticeError::IndexCorrupted { .. } | LatticeError::DatabaseError { .. } => {
            Some("rm .lattice/index.sqlite && lat check".to_string())
        }
        _ => None,
    }
}
