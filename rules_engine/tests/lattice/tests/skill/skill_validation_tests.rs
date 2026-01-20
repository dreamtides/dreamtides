use lattice::skill::skill_validation::{
    SkillValidationError, SkillValidationResult, check_description_empty, check_description_length,
    check_name_length, check_reserved_words, check_xml_characters, validate_skill,
    validate_skill_description, validate_skill_name,
};

// =============================================================================
// check_reserved_words (S001)
// =============================================================================

#[test]
fn check_reserved_words_detects_claude() {
    let result = check_reserved_words("my-claude-skill");
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "S001");
    assert!(error.message.contains("'claude'"));
}

#[test]
fn check_reserved_words_detects_anthropic() {
    let result = check_reserved_words("anthropic-tools");
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "S001");
    assert!(error.message.contains("'anthropic'"));
}

#[test]
fn check_reserved_words_case_insensitive() {
    let result = check_reserved_words("my-CLAUDE-skill");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S001");
}

#[test]
fn check_reserved_words_allows_valid_name() {
    let result = check_reserved_words("my-valid-skill");
    assert!(result.is_none());
}

#[test]
fn check_reserved_words_allows_partial_match() {
    let result = check_reserved_words("clouded-vision");
    assert!(result.is_none(), "Should not match partial word 'cloud' vs 'claude'");
}

// =============================================================================
// check_description_empty (S002)
// =============================================================================

#[test]
fn check_description_empty_detects_empty() {
    let result = check_description_empty("");
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "S002");
    assert!(error.message.contains("non-empty"));
}

#[test]
fn check_description_empty_detects_whitespace_only() {
    let result = check_description_empty("   \t\n  ");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S002");
}

#[test]
fn check_description_empty_allows_valid_description() {
    let result = check_description_empty("A valid description");
    assert!(result.is_none());
}

// =============================================================================
// check_xml_characters (S003)
// =============================================================================

#[test]
fn check_xml_characters_detects_less_than() {
    let result = check_xml_characters("my<skill");
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "S003");
    assert!(error.message.contains("XML tags"));
}

#[test]
fn check_xml_characters_detects_greater_than() {
    let result = check_xml_characters("my>skill");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S003");
}

#[test]
fn check_xml_characters_detects_full_tag() {
    let result = check_xml_characters("<tag>skill</tag>");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S003");
}

#[test]
fn check_xml_characters_allows_valid_name() {
    let result = check_xml_characters("my-valid-skill");
    assert!(result.is_none());
}

// =============================================================================
// check_name_length (W002)
// =============================================================================

#[test]
fn check_name_length_allows_64_chars() {
    let name = "a".repeat(64);
    let result = check_name_length(&name);
    assert!(result.is_none());
}

#[test]
fn check_name_length_warns_on_65_chars() {
    let name = "a".repeat(65);
    let result = check_name_length(&name);
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "W002");
    assert!(error.message.contains("65 characters"));
    assert!(error.message.contains("max: 64"));
}

// =============================================================================
// check_description_length (W003)
// =============================================================================

#[test]
fn check_description_length_allows_1024_chars() {
    let desc = "a".repeat(1024);
    let result = check_description_length(&desc);
    assert!(result.is_none());
}

#[test]
fn check_description_length_warns_on_1025_chars() {
    let desc = "a".repeat(1025);
    let result = check_description_length(&desc);
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "W003");
    assert!(error.message.contains("1025 characters"));
    assert!(error.message.contains("max: 1024"));
}

// =============================================================================
// validate_skill_name
// =============================================================================

#[test]
fn validate_skill_name_collects_all_errors() {
    let mut result = SkillValidationResult::default();
    validate_skill_name(&"a".repeat(100), &mut result);
    assert!(result.errors.iter().any(|e| e.code == "W002"));
}

#[test]
fn validate_skill_name_checks_multiple_issues() {
    let mut result = SkillValidationResult::default();
    validate_skill_name("<claude>", &mut result);
    assert_eq!(result.errors.len(), 2, "Should detect both reserved word and XML characters");
    assert!(result.errors.iter().any(|e| e.code == "S001"));
    assert!(result.errors.iter().any(|e| e.code == "S003"));
}

// =============================================================================
// validate_skill_description
// =============================================================================

#[test]
fn validate_skill_description_checks_empty_and_length() {
    let mut result = SkillValidationResult::default();
    validate_skill_description("", &mut result);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors.iter().any(|e| e.code == "S002"));
}

// =============================================================================
// validate_skill
// =============================================================================

#[test]
fn validate_skill_returns_valid_for_good_input() {
    let result = validate_skill("my-skill", "A great skill for doing things");
    assert!(result.is_valid());
    assert!(result.errors.is_empty());
}

#[test]
fn validate_skill_collects_all_validation_errors() {
    let result = validate_skill("claude", "");
    assert!(!result.is_valid());
    assert_eq!(result.errors.len(), 2, "Should have S001 and S002 errors");
}

// =============================================================================
// SkillValidationResult
// =============================================================================

#[test]
fn skill_validation_result_errors_only_filters_correctly() {
    let result = validate_skill(&"a".repeat(100), "Valid description");
    assert_eq!(result.errors_only().len(), 0, "W002 is a warning, not an error");
    assert_eq!(result.warnings_only().len(), 1, "Should have one W002 warning");
}

#[test]
fn skill_validation_result_has_errors_works() {
    let result = validate_skill("claude-skill", "Valid description");
    assert!(result.has_errors(), "S001 is a blocking error");

    let result2 = validate_skill(&"a".repeat(100), "Valid description");
    assert!(!result2.has_errors(), "W002 is just a warning");
}

// =============================================================================
// SkillValidationError
// =============================================================================

#[test]
fn skill_validation_error_is_error_distinguishes_severity() {
    let s_error = SkillValidationError { code: "S001", message: "test".to_string() };
    let w_warning = SkillValidationError { code: "W002", message: "test".to_string() };
    let e_error = SkillValidationError { code: "E001", message: "test".to_string() };

    assert!(s_error.is_error());
    assert!(!w_warning.is_error());
    assert!(e_error.is_error());
}
