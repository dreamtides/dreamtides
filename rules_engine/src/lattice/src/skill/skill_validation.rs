use crate::document::frontmatter_schema::{MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH};

/// Reserved words that cannot appear in skill names (case-insensitive).
pub const RESERVED_WORDS: &[&str] = &["anthropic", "claude"];

/// A single skill validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillValidationError {
    /// The lint code associated with this error (e.g., "S001").
    pub code: &'static str,
    /// Human-readable message describing the validation failure.
    pub message: String,
}

/// Result of validating a skill document's fields.
#[derive(Debug, Clone, Default)]
pub struct SkillValidationResult {
    /// List of validation errors found.
    pub errors: Vec<SkillValidationError>,
}

/// Validates all skill-specific constraints for a document.
///
/// Checks S001 (reserved words), S002 (empty description), and S003 (XML
/// characters), as well as max length constraints. Returns a result containing
/// all validation errors found.
pub fn validate_skill(name: &str, description: &str) -> SkillValidationResult {
    let mut result = SkillValidationResult::default();
    validate_skill_name(name, &mut result);
    validate_skill_description(description, &mut result);
    result
}

/// Validates the skill name field against all skill-specific rules.
///
/// Checks:
/// - S001: Name must not contain reserved words ("anthropic", "claude")
/// - S003: Name must not contain XML-like characters (< or >)
/// - Name must not exceed 64 characters
pub fn validate_skill_name(name: &str, result: &mut SkillValidationResult) {
    if let Some(error) = check_reserved_words(name) {
        result.errors.push(error);
    }
    if let Some(error) = check_xml_characters(name) {
        result.errors.push(error);
    }
    if let Some(error) = check_name_length(name) {
        result.errors.push(error);
    }
}

/// Validates the skill description field against all skill-specific rules.
///
/// Checks:
/// - S002: Description must not be empty
/// - Description must not exceed 1024 characters
pub fn validate_skill_description(description: &str, result: &mut SkillValidationResult) {
    if let Some(error) = check_description_empty(description) {
        result.errors.push(error);
    }
    if let Some(error) = check_description_length(description) {
        result.errors.push(error);
    }
}

/// S001: Checks if name contains any reserved words.
///
/// Returns an error if the name contains "anthropic" or "claude"
/// (case-insensitive).
pub fn check_reserved_words(name: &str) -> Option<SkillValidationError> {
    let name_lower = name.to_lowercase();
    for reserved in RESERVED_WORDS {
        if name_lower.contains(reserved) {
            tracing::debug!(name, reserved, "Skill name contains reserved word");
            return Some(SkillValidationError {
                code: "S001",
                message: format!("skill name cannot contain '{reserved}'"),
            });
        }
    }
    None
}

/// S002: Checks if description is empty or whitespace-only.
///
/// Returns an error if the description is empty after trimming whitespace.
pub fn check_description_empty(description: &str) -> Option<SkillValidationError> {
    if description.trim().is_empty() {
        tracing::debug!("Skill has empty description");
        return Some(SkillValidationError {
            code: "S002",
            message: "skill must have non-empty description".to_string(),
        });
    }
    None
}

/// S003: Checks if name contains XML-like characters.
///
/// Returns an error if the name contains '<' or '>'.
pub fn check_xml_characters(name: &str) -> Option<SkillValidationError> {
    if name.contains('<') || name.contains('>') {
        tracing::debug!(name, "Skill name contains XML characters");
        return Some(SkillValidationError {
            code: "S003",
            message: "skill name cannot contain XML tags".to_string(),
        });
    }
    None
}

/// Checks if name exceeds the maximum length.
///
/// Returns an error if the name is longer than 64 characters.
pub fn check_name_length(name: &str) -> Option<SkillValidationError> {
    if name.len() > MAX_NAME_LENGTH {
        tracing::debug!(
            name_len = name.len(),
            max = MAX_NAME_LENGTH,
            "Skill name exceeds maximum length"
        );
        return Some(SkillValidationError {
            code: "W002",
            message: format!("name is {} characters (max: {})", name.len(), MAX_NAME_LENGTH),
        });
    }
    None
}

/// Checks if description exceeds the maximum length.
///
/// Returns an error if the description is longer than 1024 characters.
pub fn check_description_length(description: &str) -> Option<SkillValidationError> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        tracing::debug!(
            desc_len = description.len(),
            max = MAX_DESCRIPTION_LENGTH,
            "Skill description exceeds maximum length"
        );
        return Some(SkillValidationError {
            code: "W003",
            message: format!(
                "description is {} characters (max: {})",
                description.len(),
                MAX_DESCRIPTION_LENGTH
            ),
        });
    }
    None
}

impl SkillValidationResult {
    /// Returns true if validation passed with no errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns only errors (codes starting with 'S' or 'E').
    pub fn errors_only(&self) -> Vec<&SkillValidationError> {
        self.errors.iter().filter(|e| e.code.starts_with('S') || e.code.starts_with('E')).collect()
    }

    /// Returns only warnings (codes starting with 'W').
    pub fn warnings_only(&self) -> Vec<&SkillValidationError> {
        self.errors.iter().filter(|e| e.code.starts_with('W')).collect()
    }

    /// Returns true if there are any blocking errors (S-codes).
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.code.starts_with('S') || e.code.starts_with('E'))
    }
}

impl SkillValidationError {
    /// Returns true if this is a blocking error (not a warning).
    pub fn is_error(&self) -> bool {
        self.code.starts_with('S') || self.code.starts_with('E')
    }
}
