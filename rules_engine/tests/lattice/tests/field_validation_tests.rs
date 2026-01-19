use std::path::Path;

use lattice::document::field_validation;
use lattice::document::frontmatter_schema::{
    Frontmatter, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH, TaskType,
};
use lattice::error::error_types::LatticeError;
use lattice::id::lattice_id::LatticeId;

fn sample_lattice_id() -> LatticeId {
    "LABCDT".parse().expect("Valid test ID")
}

fn sample_task_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: sample_lattice_id(),
        name: "fix-login-bug".to_string(),
        description: "Fix login after password reset".to_string(),
        parent_id: None,
        task_type: Some(TaskType::Bug),
        priority: Some(1),
        labels: vec![],
        blocking: vec![],
        blocked_by: vec![],
        discovered_from: vec![],
        created_at: None,
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

fn sample_kb_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: sample_lattice_id(),
        name: "auth-design".to_string(),
        description: "Authentication system design".to_string(),
        parent_id: None,
        task_type: None,
        priority: None,
        labels: vec![],
        blocking: vec![],
        blocked_by: vec![],
        discovered_from: vec![],
        created_at: None,
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

// =============================================================================
// Name Validation - Valid Names
// =============================================================================

#[test]
fn accepts_valid_lowercase_name() {
    let fm = sample_kb_frontmatter();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(result.is_valid(), "Should accept valid lowercase name: {:?}", result.errors);
}

#[test]
fn accepts_name_with_hyphens() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "fix-login-bug".to_string();
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(result.is_valid(), "Should accept name with hyphens: {:?}", result.errors);
}

#[test]
fn accepts_name_with_numbers() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "auth-v2".to_string();
    let result = field_validation::validate(&fm, Path::new("auth_v2.md"));

    assert!(result.is_valid(), "Should accept name with numbers: {:?}", result.errors);
}

#[test]
fn accepts_single_word_name() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "readme".to_string();
    let result = field_validation::validate(&fm, Path::new("readme.md"));

    assert!(result.is_valid(), "Should accept single word name: {:?}", result.errors);
}

// =============================================================================
// Name Validation - Invalid Names
// =============================================================================

#[test]
fn rejects_empty_name() {
    let mut fm = sample_kb_frontmatter();
    fm.name = String::new();
    let result = field_validation::validate(&fm, Path::new("test.md"));

    assert!(!result.is_valid(), "Should reject empty name");
    assert!(
        result.errors.iter().any(|e| e.field == "name" && e.reason.contains("cannot be empty")),
        "Should have name empty error: {:?}",
        result.errors
    );
}

#[test]
fn rejects_name_with_uppercase() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "Auth-Design".to_string();
    let result = field_validation::validate(&fm, Path::new("Auth_Design.md"));

    assert!(!result.is_valid(), "Should reject name with uppercase");
    assert!(
        result.errors.iter().any(|e| e.field == "name" && e.reason.contains("lowercase")),
        "Should mention lowercase requirement: {:?}",
        result.errors
    );
}

#[test]
fn rejects_name_with_underscores() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "auth_design".to_string();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(!result.is_valid(), "Should reject name with underscores");
    assert!(
        result.errors.iter().any(|e| e.field == "name" && e.reason.contains("lowercase")),
        "Should mention valid characters: {:?}",
        result.errors
    );
}

#[test]
fn rejects_name_with_spaces() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "auth design".to_string();
    let result = field_validation::validate(&fm, Path::new("auth design.md"));

    assert!(!result.is_valid(), "Should reject name with spaces");
}

#[test]
fn rejects_name_starting_with_hyphen() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "-auth-design".to_string();
    let result = field_validation::validate(&fm, Path::new("-auth_design.md"));

    assert!(!result.is_valid(), "Should reject name starting with hyphen");
}

#[test]
fn rejects_name_ending_with_hyphen() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "auth-design-".to_string();
    let result = field_validation::validate(&fm, Path::new("auth_design_.md"));

    assert!(!result.is_valid(), "Should reject name ending with hyphen");
}

#[test]
fn rejects_name_exceeding_max_length() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "a".repeat(MAX_NAME_LENGTH + 1);
    let result = field_validation::validate(&fm, Path::new("test.md"));

    assert!(!result.is_valid(), "Should reject name exceeding max length");
    assert!(
        result.errors.iter().any(|e| e.field == "name" && e.reason.contains("maximum length")),
        "Should mention maximum length: {:?}",
        result.errors
    );
}

#[test]
fn accepts_name_at_max_length() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "a".repeat(MAX_NAME_LENGTH);
    let result = field_validation::validate(&fm, Path::new(&format!("{}.md", fm.name)));

    assert!(result.is_valid(), "Should accept name at max length: {:?}", result.errors);
}

// =============================================================================
// Name-Filename Mismatch (E008)
// =============================================================================

#[test]
fn rejects_name_not_matching_filename() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "different-name".to_string();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(!result.is_valid(), "Should reject name not matching filename");
    assert!(
        result.errors.iter().any(|e| e.field == "name" && e.reason.contains("must match filename")),
        "Should mention filename mismatch: {:?}",
        result.errors
    );
}

#[test]
fn matches_name_with_underscore_to_hyphen_conversion() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "fix-login-bug".to_string();
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(result.is_valid(), "Should match when underscores become hyphens: {:?}", result.errors);
}

#[test]
fn derives_name_from_path_correctly() {
    assert_eq!(
        field_validation::derive_name_from_path(Path::new("Fix_Login_Bug.md")),
        Some("fix-login-bug".to_string()),
        "Should lowercase and convert underscores"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("dir/subdir/Auth_V2.md")),
        Some("auth-v2".to_string()),
        "Should handle nested paths"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("simple.md")),
        Some("simple".to_string()),
        "Should handle simple filename"
    );
}

#[test]
fn derives_name_stripping_lattice_id_suffix() {
    assert_eq!(
        field_validation::derive_name_from_path(Path::new("fix_login_bug_LABCDEF.md")),
        Some("fix-login-bug".to_string()),
        "Should strip trailing lattice ID suffix"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("tasks/verify_task_filing_LCSWQN.md")),
        Some("verify-task-filing".to_string()),
        "Should strip lattice ID from nested path"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("my_task_labcdef.md")),
        Some("my-task".to_string()),
        "Should strip lowercase lattice ID suffix"
    );
}

#[test]
fn derives_name_preserves_non_id_suffixes() {
    assert_eq!(
        field_validation::derive_name_from_path(Path::new("task_v2.md")),
        Some("task-v2".to_string()),
        "Should not strip short suffixes that look like IDs but are too short"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("task_ABC.md")),
        Some("task-abc".to_string()),
        "Should not strip suffix missing L prefix"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("task_L12.md")),
        Some("task-l12".to_string()),
        "Should not strip suffix too short to be valid ID"
    );

    assert_eq!(
        field_validation::derive_name_from_path(Path::new("process_log_file.md")),
        Some("process-log-file".to_string()),
        "Should not strip words starting with L that aren't IDs"
    );
}

#[test]
fn accepts_name_with_lattice_id_suffix_in_filename() {
    let mut fm = sample_kb_frontmatter();
    fm.name = "fix-login-bug".to_string();
    let result = field_validation::validate(&fm, Path::new("fix_login_bug_LABCDEF.md"));

    assert!(
        result.is_valid(),
        "Should accept name matching filename with lattice ID stripped: {:?}",
        result.errors
    );
}

// =============================================================================
// Description Validation
// =============================================================================

#[test]
fn accepts_valid_description() {
    let fm = sample_kb_frontmatter();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(result.is_valid(), "Should accept valid description: {:?}", result.errors);
}

#[test]
fn rejects_empty_description() {
    let mut fm = sample_kb_frontmatter();
    fm.description = String::new();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(!result.is_valid(), "Should reject empty description");
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.field == "description" && e.reason.contains("cannot be empty")),
        "Should mention empty description: {:?}",
        result.errors
    );
}

#[test]
fn rejects_description_exceeding_max_length() {
    let mut fm = sample_kb_frontmatter();
    fm.description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(!result.is_valid(), "Should reject description exceeding max length");
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.field == "description" && e.reason.contains("maximum length")),
        "Should mention maximum length: {:?}",
        result.errors
    );
}

#[test]
fn accepts_description_at_max_length() {
    let mut fm = sample_kb_frontmatter();
    fm.description = "a".repeat(MAX_DESCRIPTION_LENGTH);
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(result.is_valid(), "Should accept description at max length: {:?}", result.errors);
}

// =============================================================================
// Priority Validation
// =============================================================================

#[test]
fn accepts_valid_priority_for_task() {
    for priority in 0..=4 {
        let mut fm = sample_task_frontmatter();
        fm.priority = Some(priority);
        let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

        assert!(result.is_valid(), "Should accept priority {}: {:?}", priority, result.errors);
    }
}

#[test]
fn rejects_priority_exceeding_max() {
    let mut fm = sample_task_frontmatter();
    fm.priority = Some(5);
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(!result.is_valid(), "Should reject priority > 4");
    assert!(
        result.errors.iter().any(|e| e.field == "priority" && e.reason.contains("between")),
        "Should mention valid range: {:?}",
        result.errors
    );
}

#[test]
fn rejects_missing_priority_for_task() {
    let mut fm = sample_task_frontmatter();
    fm.priority = None;
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(!result.is_valid(), "Should reject missing priority for task");
    assert!(
        result.errors.iter().any(|e| e.field == "priority" && e.reason.contains("required")),
        "Should mention priority required: {:?}",
        result.errors
    );
}

#[test]
fn accepts_missing_priority_for_kb_document() {
    let fm = sample_kb_frontmatter();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(result.is_valid(), "Should accept KB doc without priority: {:?}", result.errors);
}

// =============================================================================
// Batch Validation (Multiple Errors)
// =============================================================================

#[test]
fn collects_multiple_errors() {
    let mut fm = sample_task_frontmatter();
    fm.name = String::new();
    fm.description = String::new();
    fm.priority = None;
    let result = field_validation::validate(&fm, Path::new("test.md"));

    assert!(!result.is_valid(), "Should be invalid with multiple errors");
    assert!(result.errors.len() >= 3, "Should collect multiple errors: {:?}", result.errors);
    assert!(
        result.errors.iter().any(|e| e.field == "name"),
        "Should have name error: {:?}",
        result.errors
    );
    assert!(
        result.errors.iter().any(|e| e.field == "description"),
        "Should have description error: {:?}",
        result.errors
    );
    assert!(
        result.errors.iter().any(|e| e.field == "priority"),
        "Should have priority error: {:?}",
        result.errors
    );
}

#[test]
fn to_error_combines_all_errors() {
    let mut fm = sample_task_frontmatter();
    fm.name = String::new();
    fm.description = String::new();
    let result = field_validation::validate(&fm, Path::new("test.md"));

    let error = result.to_error(Path::new("test.md"));
    assert!(error.is_some(), "Should produce error");

    if let Some(LatticeError::InvalidFrontmatter { reason, .. }) = error {
        assert!(reason.contains("name"), "Combined error should mention name: {reason}");
        assert!(
            reason.contains("description"),
            "Combined error should mention description: {reason}"
        );
    } else {
        panic!("Expected InvalidFrontmatter error");
    }
}

#[test]
fn to_error_returns_none_for_valid() {
    let fm = sample_kb_frontmatter();
    let result = field_validation::validate(&fm, Path::new("auth_design.md"));

    assert!(result.to_error(Path::new("auth_design.md")).is_none(), "Should return None for valid");
}

// =============================================================================
// Standalone Validation Functions
// =============================================================================

#[test]
fn validate_name_only_accepts_valid() {
    assert!(field_validation::validate_name_only("valid-name").is_ok(), "Should accept valid name");
}

#[test]
fn validate_name_only_rejects_invalid() {
    let result = field_validation::validate_name_only("Invalid_Name");
    assert!(result.is_err(), "Should reject invalid name");

    if let Err(LatticeError::InvalidFieldValue { field, reason, .. }) = result {
        assert_eq!(field, "name");
        assert!(reason.contains("lowercase"), "Should mention requirement: {reason}");
    } else {
        panic!("Expected InvalidFieldValue error");
    }
}

#[test]
fn validate_description_only_accepts_valid() {
    assert!(
        field_validation::validate_description_only("A valid description").is_ok(),
        "Should accept valid description"
    );
}

#[test]
fn validate_description_only_rejects_empty() {
    let result = field_validation::validate_description_only("");
    assert!(result.is_err(), "Should reject empty description");

    if let Err(LatticeError::InvalidFieldValue { field, .. }) = result {
        assert_eq!(field, "description");
    } else {
        panic!("Expected InvalidFieldValue error");
    }
}

#[test]
fn validate_priority_only_accepts_valid() {
    for p in 0..=4 {
        assert!(field_validation::validate_priority_only(p).is_ok(), "Should accept priority {p}");
    }
}

#[test]
fn validate_priority_only_rejects_invalid() {
    let result = field_validation::validate_priority_only(5);
    assert!(result.is_err(), "Should reject priority 5");

    if let Err(LatticeError::InvalidFieldValue { field, reason, .. }) = result {
        assert_eq!(field, "priority");
        assert!(reason.contains("between"), "Should mention valid range: {reason}");
    } else {
        panic!("Expected InvalidFieldValue error");
    }
}

// =============================================================================
// ID Reference Validation
// =============================================================================

#[test]
fn accepts_valid_id_references() {
    let mut fm = sample_task_frontmatter();
    fm.blocking = vec!["LB234A".parse().unwrap()];
    fm.blocked_by = vec!["LC567B".parse().unwrap()];
    fm.discovered_from = vec!["LD234C".parse().unwrap()];
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(result.is_valid(), "Should accept valid ID references: {:?}", result.errors);
}

#[test]
fn accepts_empty_id_reference_lists() {
    let fm = sample_task_frontmatter();
    let result = field_validation::validate(&fm, Path::new("fix_login_bug.md"));

    assert!(result.is_valid(), "Should accept empty ID reference lists: {:?}", result.errors);
}
