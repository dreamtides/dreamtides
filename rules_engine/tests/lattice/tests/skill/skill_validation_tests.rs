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

#[test]
fn check_reserved_words_detects_embedded_claude() {
    let result = check_reserved_words("my-claude-powered-tool");
    assert!(result.is_some(), "Should detect 'claude' embedded in string");
    assert_eq!(result.unwrap().code, "S001");
}

#[test]
fn check_reserved_words_detects_mixed_case_anthropic() {
    let result = check_reserved_words("AnThRoPiC-sdk");
    assert!(result.is_some(), "Should detect mixed case 'anthropic'");
    assert_eq!(result.unwrap().code, "S001");
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

#[test]
fn check_description_empty_allows_minimal_valid_description() {
    let result = check_description_empty("X");
    assert!(result.is_none(), "Single character description should be valid");
}

#[test]
fn check_description_empty_detects_newlines_only() {
    let result = check_description_empty("\n\n\n");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S002");
}

#[test]
fn check_description_empty_detects_tabs_and_spaces() {
    let result = check_description_empty("\t  \t  ");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S002");
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

#[test]
fn check_xml_characters_allows_hyphens_and_underscores() {
    let result = check_xml_characters("my_skill-name_v2");
    assert!(result.is_none());
}

#[test]
fn check_xml_characters_detects_self_closing_tag() {
    let result = check_xml_characters("my-<br/>-skill");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S003");
}

#[test]
fn check_xml_characters_detects_only_less_than() {
    let result = check_xml_characters("value<10");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S003");
}

#[test]
fn check_xml_characters_detects_only_greater_than() {
    let result = check_xml_characters("value>10");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code, "S003");
}

// =============================================================================
// check_name_length (W002)
// =============================================================================

#[test]
fn check_name_length_allows_63_chars() {
    let name = "a".repeat(63);
    let result = check_name_length(&name);
    assert!(result.is_none(), "63 chars should be within limit");
}

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

#[test]
fn check_name_length_allows_empty_name() {
    let result = check_name_length("");
    assert!(result.is_none(), "Empty name passes length check (other validations catch empty)");
}

// =============================================================================
// check_description_length (W003)
// =============================================================================

#[test]
fn check_description_length_allows_1023_chars() {
    let desc = "a".repeat(1023);
    let result = check_description_length(&desc);
    assert!(result.is_none(), "1023 chars should be within limit");
}

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

#[test]
fn check_description_length_warns_on_very_long_description() {
    let desc = "a".repeat(5000);
    let result = check_description_length(&desc);
    assert!(result.is_some());
    let error = result.unwrap();
    assert_eq!(error.code, "W003");
    assert!(error.message.contains("5000 characters"));
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

#[test]
fn validate_skill_with_all_errors_at_once() {
    // Name has: reserved word (S001), XML char (S003), over length (W002)
    // Description has: empty (S002), and if we pass a long empty-ish string with <
    // we get length warning
    let long_name_with_issues = format!("<claude>{}", "x".repeat(70));
    let result = validate_skill(&long_name_with_issues, "");

    assert!(!result.is_valid());
    // Should have S001 (reserved), S003 (XML), W002 (length), S002 (empty desc)
    assert!(result.errors.iter().any(|e| e.code == "S001"), "Should have reserved word error");
    assert!(result.errors.iter().any(|e| e.code == "S003"), "Should have XML char error");
    assert!(result.errors.iter().any(|e| e.code == "W002"), "Should have name length warning");
    assert!(result.errors.iter().any(|e| e.code == "S002"), "Should have empty description error");
}

#[test]
fn validate_skill_valid_at_max_lengths() {
    let name = "x".repeat(64);
    let description = "y".repeat(1024);
    let result = validate_skill(&name, &description);
    assert!(result.is_valid(), "Max-length name and description should be valid");
    assert!(result.errors.is_empty());
}

#[test]
fn validate_skill_warnings_only_when_lengths_exceeded() {
    let name = "x".repeat(100);
    let description = "y".repeat(2000);
    let result = validate_skill(&name, &description);

    // These are just warnings, not blocking errors
    assert!(!result.has_errors(), "Length issues are warnings, not errors");
    assert_eq!(result.warnings_only().len(), 2, "Should have two length warnings");
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
