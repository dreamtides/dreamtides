use std::path::Path;

use crate::document::frontmatter_schema::{
    Frontmatter, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH, MAX_PRIORITY, MIN_PRIORITY,
};
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::MIN_ID_LENGTH;

/// A single field validation error with field name and reason.
#[derive(Debug, Clone)]
pub struct FieldError {
    /// Name of the field that failed validation.
    pub field: String,
    /// Human-readable explanation of what went wrong.
    pub reason: String,
}

/// Result of validating all fields in a frontmatter document.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// List of validation errors found.
    pub errors: Vec<FieldError>,
}

/// Validates all fields in frontmatter, collecting all errors.
///
/// This performs semantic validation beyond what YAML parsing provides.
/// It checks field values for correctness and consistency.
pub fn validate(frontmatter: &Frontmatter, path: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();

    validate_name(&frontmatter.name, &mut result);
    validate_name_matches_filename(&frontmatter.name, path, &mut result);
    validate_description(&frontmatter.description, &mut result);
    validate_priority(frontmatter, &mut result);
    // Note: ID references (blocking, blocked-by, discovered-from) are validated
    // at parse time by LatticeId's Deserialize implementation. Cross-reference
    // validation (checking if target IDs exist) is done by the linter.

    tracing::debug!(
        path = %path.display(),
        errors = result.errors.len(),
        "Field validation complete"
    );

    result
}

/// Derives the expected name from a file path.
///
/// Converts filename to name format: strips `.md` extension, strips any
/// trailing lattice ID suffix (e.g., `_LABCDEF`), converts underscores to
/// hyphens, and lowercases. This allows filenames like `my_task_LABCDEF.md` to
/// produce name `my-task`.
pub fn derive_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let stem_without_id = strip_lattice_id_suffix(stem);
    Some(stem_without_id.to_lowercase().replace('_', "-"))
}

/// Validates a single name string without path context.
///
/// Useful for validating name values before document creation.
pub fn validate_name_only(name: &str) -> Result<(), LatticeError> {
    let mut result = ValidationResult::new();
    validate_name(name, &mut result);

    if let Some(error) = result.errors.into_iter().next() {
        return Err(LatticeError::InvalidFieldValue {
            field: error.field,
            path: std::path::PathBuf::new(),
            reason: error.reason,
        });
    }

    Ok(())
}

/// Validates a single description string.
///
/// Useful for validating description values before document creation.
pub fn validate_description_only(description: &str) -> Result<(), LatticeError> {
    let mut result = ValidationResult::new();
    validate_description(description, &mut result);

    if let Some(error) = result.errors.into_iter().next() {
        return Err(LatticeError::InvalidFieldValue {
            field: error.field,
            path: std::path::PathBuf::new(),
            reason: error.reason,
        });
    }

    Ok(())
}

/// Validates a priority value.
///
/// Useful for validating priority values before document creation.
pub fn validate_priority_only(priority: u8) -> Result<(), LatticeError> {
    if priority > MAX_PRIORITY {
        return Err(LatticeError::InvalidFieldValue {
            field: "priority".to_string(),
            path: std::path::PathBuf::new(),
            reason: format!(
                "must be between {} and {} (got {})",
                MIN_PRIORITY, MAX_PRIORITY, priority
            ),
        });
    }

    Ok(())
}

/// Strips a trailing lattice ID suffix from a filename stem.
///
/// Lattice IDs start with 'L' followed by Base32 characters (A-Z, 2-7).
/// If the stem ends with `_L{Base32}`, the suffix is stripped.
fn strip_lattice_id_suffix(stem: &str) -> &str {
    if let Some(last_underscore) = stem.rfind('_') {
        let potential_id = &stem[last_underscore + 1..];
        if looks_like_lattice_id(potential_id) {
            return &stem[..last_underscore];
        }
    }
    stem
}

/// Checks if a string looks like a valid Lattice ID.
fn looks_like_lattice_id(s: &str) -> bool {
    if s.len() < MIN_ID_LENGTH {
        return false;
    }
    let first = s.chars().next().unwrap_or('\0');
    if first != 'L' && first != 'l' {
        return false;
    }
    s[1..]
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_lowercase() || ('2'..='7').contains(&c))
}

impl FieldError {
    fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { field: field.into(), reason: reason.into() }
    }
}

impl ValidationResult {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn add_error(&mut self, error: FieldError) {
        self.errors.push(error);
    }

    /// Returns true if validation passed with no errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Converts to a `LatticeError` if there are any errors.
    pub fn to_error(&self, path: &Path) -> Option<LatticeError> {
        if self.errors.is_empty() {
            return None;
        }

        let reasons: Vec<String> =
            self.errors.iter().map(|e| format!("{}: {}", e.field, e.reason)).collect();

        Some(LatticeError::InvalidFrontmatter {
            id: String::new(),
            path: path.to_path_buf(),
            reason: reasons.join("; "),
        })
    }
}

/// Validates that the name field follows the required format.
fn validate_name(name: &str, result: &mut ValidationResult) {
    if name.is_empty() {
        result.add_error(FieldError::new("name", "cannot be empty"));
        return;
    }

    if name.len() > MAX_NAME_LENGTH {
        result.add_error(FieldError::new(
            "name",
            format!(
                "exceeds maximum length of {} characters (got {})",
                MAX_NAME_LENGTH,
                name.len()
            ),
        ));
    }

    if !is_valid_name_format(name) {
        result.add_error(FieldError::new(
            "name",
            "must contain only lowercase letters, numbers, and hyphens",
        ));
    }
}

/// Checks if a name contains only lowercase letters, numbers, and hyphens.
fn is_valid_name_format(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Validates that the name field matches the filename.
fn validate_name_matches_filename(name: &str, path: &Path, result: &mut ValidationResult) {
    let Some(expected) = derive_name_from_path(path) else {
        return;
    };

    if name != expected {
        result.add_error(FieldError::new(
            "name",
            format!("must match filename (expected '{}', got '{}')", expected, name),
        ));
    }
}

/// Validates the description field.
fn validate_description(description: &str, result: &mut ValidationResult) {
    if description.is_empty() {
        result.add_error(FieldError::new("description", "cannot be empty"));
        return;
    }

    if description.len() > MAX_DESCRIPTION_LENGTH {
        result.add_error(FieldError::new(
            "description",
            format!(
                "exceeds maximum length of {} characters (got {})",
                MAX_DESCRIPTION_LENGTH,
                description.len()
            ),
        ));
    }
}

/// Validates the priority field for tasks.
fn validate_priority(frontmatter: &Frontmatter, result: &mut ValidationResult) {
    if frontmatter.task_type.is_some() && frontmatter.priority.is_none() {
        result.add_error(FieldError::new("priority", "required for task documents"));
    }

    if let Some(priority) = frontmatter.priority
        && priority > MAX_PRIORITY
    {
        result.add_error(FieldError::new(
            "priority",
            format!("must be between {} and {} (got {})", MIN_PRIORITY, MAX_PRIORITY, priority),
        ));
    }
}
