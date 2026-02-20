use parser_tests::test_helpers;

const BRACKET_LANGUAGE: &str = "en-x-bracket";
const SOURCE_LANGUAGE: &str = "en";

/// Validates that every English source phrase has a corresponding definition
/// in the bracket locale, ensuring bracket locale completeness.
#[test]
fn bracket_locale_covers_all_source_phrases() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    let missing = rlf::with_locale(|locale| {
        let source = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered");
        let bracket =
            locale.registry_for(BRACKET_LANGUAGE).expect("bracket locale should be registered");

        let mut missing_phrases = Vec::new();
        let mut source_names: Vec<&str> = source.phrase_names().collect();
        source_names.sort();
        for name in source_names {
            if bracket.get(name).is_none() {
                missing_phrases.push(name.to_string());
            }
        }
        missing_phrases
    });
    assert!(
        missing.is_empty(),
        "bracket locale is missing {} phrase(s) present in English source:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

/// Validates that the bracket locale does not define any phrases absent
/// from the English source (no orphan phrases).
#[test]
fn bracket_locale_has_no_orphan_phrases() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    let orphans = rlf::with_locale(|locale| {
        let source = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered");
        let bracket =
            locale.registry_for(BRACKET_LANGUAGE).expect("bracket locale should be registered");

        let mut orphan_phrases = Vec::new();
        let mut bracket_names: Vec<&str> = bracket.phrase_names().collect();
        bracket_names.sort();
        for name in bracket_names {
            if source.get(name).is_none() {
                orphan_phrases.push(name.to_string());
            }
        }
        orphan_phrases
    });
    assert!(
        orphans.is_empty(),
        "bracket locale has {} orphan phrase(s) not in English source:\n  {}",
        orphans.len(),
        orphans.join("\n  ")
    );
}

/// Validates that the bracket locale has the same parameter counts as the
/// English source for every phrase.
#[test]
fn bracket_locale_parameter_counts_match_source() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    let mismatches = rlf::with_locale(|locale| {
        let source = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered");
        let bracket =
            locale.registry_for(BRACKET_LANGUAGE).expect("bracket locale should be registered");

        let mut issues = Vec::new();
        let mut source_names: Vec<&str> = source.phrase_names().collect();
        source_names.sort();
        for name in source_names {
            let Some(source_def) = source.get(name) else {
                continue;
            };
            let Some(bracket_def) = bracket.get(name) else {
                continue;
            };
            if source_def.parameters.len() != bracket_def.parameters.len() {
                issues.push(format!(
                    "{name}: source has {} param(s), bracket has {}",
                    source_def.parameters.len(),
                    bracket_def.parameters.len()
                ));
            }
        }
        issues
    });
    assert!(
        mismatches.is_empty(),
        "bracket locale has {} parameter count mismatch(es):\n  {}",
        mismatches.len(),
        mismatches.join("\n  ")
    );
}

// =========================================================================
// Reachable-graph checks: missing variant and tag coverage.
//
// These tests use standalone Locale instances to avoid global state
// interference. They demonstrate that the RLF evaluator correctly
// fails when a phrase selects a variant or requires a tag that does
// not exist on the provided parameter value.
// =========================================================================

/// Validates that evaluate-time errors occur when a phrase selects a
/// variant that does not exist on its parameter.
#[test]
fn missing_variant_coverage_causes_eval_error() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
                target_noun = { one: "warrior", *other: "warriors" };
                dissolve_target($t) = "dissolve {$t:acc}";
            "#,
        )
        .expect("loading test phrases should succeed");

    let noun = locale.get_phrase("target_noun").expect("target_noun should resolve");
    let result = locale.call_phrase("dissolve_target", &[rlf::Value::Phrase(noun)]);
    assert!(result.is_err(), "selecting a nonexistent variant should produce an eval error");
    let error_str = result.unwrap_err().to_string();
    assert!(
        error_str.contains("missing variant") || error_str.contains("acc"),
        "error should indicate missing variant 'acc', got: {error_str}"
    );
}

/// Validates that evaluate-time errors occur when a transform requires a
/// tag that the phrase does not carry.
#[test]
fn missing_tag_coverage_causes_eval_error() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
                untagged_noun = "warrior";
                dissolve_target($t) = "dissolve {@a $t}";
            "#,
        )
        .expect("loading test phrases should succeed");

    let noun = locale.get_phrase("untagged_noun").expect("untagged_noun should resolve");
    let result = locale.call_phrase("dissolve_target", &[rlf::Value::Phrase(noun)]);
    assert!(
        result.is_err(),
        "applying @a transform to a phrase without :a/:an tags should produce an eval error"
    );
    let error_str = result.unwrap_err().to_string();
    assert!(
        error_str.contains("tag") || error_str.contains("@a"),
        "error should indicate missing tag, got: {error_str}"
    );
}

/// Validates that a phrase with correct variant coverage evaluates
/// successfully.
#[test]
fn correct_variant_coverage_succeeds() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
                target_noun = { one: "warrior", *other: "warriors" };
                dissolve_target($t) = "dissolve {$t:one}";
            "#,
        )
        .expect("loading test phrases should succeed");

    let noun = locale.get_phrase("target_noun").expect("target_noun should resolve");
    let result = locale.call_phrase("dissolve_target", &[rlf::Value::Phrase(noun)]);
    assert!(
        result.is_ok(),
        "selecting an existing variant should succeed, got error: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().to_string(), "dissolve warrior");
}

/// Validates that a phrase with correct tag coverage evaluates
/// successfully.
#[test]
fn correct_tag_coverage_succeeds() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
                tagged_noun = :a "warrior";
                dissolve_target($t) = "dissolve {@a $t}";
            "#,
        )
        .expect("loading test phrases should succeed");

    let noun = locale.get_phrase("tagged_noun").expect("tagged_noun should resolve");
    let result = locale.call_phrase("dissolve_target", &[rlf::Value::Phrase(noun)]);
    assert!(
        result.is_ok(),
        "applying @a to a phrase with :a tag should succeed, got error: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().to_string(), "dissolve a warrior");
}

/// End-to-end validation: the bracket locale has complete coverage
/// of all source phrases with matching phrase counts.
#[test]
fn translation_validation_gate_integration() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");

    let (source_count, bracket_count) = rlf::with_locale(|locale| {
        let source = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered");
        let bracket =
            locale.registry_for(BRACKET_LANGUAGE).expect("bracket locale should be registered");
        (source.phrase_names().count(), bracket.phrase_names().count())
    });
    assert_eq!(
        source_count, bracket_count,
        "translation validation gate: source has {source_count} phrases, bracket has {bracket_count}"
    );
}
