use parser_v2_tests::test_helpers;

const BRACKET_LANGUAGE: &str = "en-x-bracket";
const SOURCE_LANGUAGE: &str = "en";

/// Validates that the bracket locale has no warnings when checked against
/// the English source phrases via `Locale::validate_translations`.
#[test]
fn bracket_locale_passes_translation_validation() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    let warnings =
        rlf::with_locale(|locale| locale.validate_translations(SOURCE_LANGUAGE, BRACKET_LANGUAGE));
    assert!(
        warnings.is_empty(),
        "bracket locale should have zero validation warnings, found {}:\n{}",
        warnings.len(),
        warnings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
    );
}

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
// Seeded bad data tests: prove that validation catches errors.
//
// These tests use unique language codes to avoid clobbering the shared
// global locale English source.
// =========================================================================

/// Seeded test: a translation with a phrase not in the source triggers an
/// `UnknownPhrase` warning.
#[test]
fn seeded_unknown_phrase_detected() {
    strings::strings::register_source_phrases();
    let warnings = rlf::with_locale_mut(|locale| {
        locale
            .load_translations_str(
                "en-x-bad-unknown",
                r#"
                    card = "carta";
                    nonexistent_phrase_xyz = "this should not exist";
                "#,
            )
            .expect("loading seeded bad locale should succeed");
        locale.validate_translations(SOURCE_LANGUAGE, "en-x-bad-unknown")
    });
    let has_unknown = warnings.iter().any(|w| {
        matches!(w, rlf::LoadWarning::UnknownPhrase { name, .. } if name == "nonexistent_phrase_xyz")
    });
    assert!(
        has_unknown,
        "validation should detect the unknown phrase 'nonexistent_phrase_xyz', got warnings: {warnings:?}"
    );
}

/// Seeded test: a translation with wrong parameter count triggers a
/// `ParameterCountMismatch` warning.
#[test]
fn seeded_parameter_count_mismatch_detected() {
    strings::strings::register_source_phrases();
    let warnings = rlf::with_locale_mut(|locale| {
        locale
            .load_translations_str(
                "en-x-bad-params",
                r#"
                    energy($e, $extra) = "<color=#00838F>{$e}{$extra}</color>";
                "#,
            )
            .expect("loading seeded bad locale should succeed");
        locale.validate_translations(SOURCE_LANGUAGE, "en-x-bad-params")
    });
    let has_mismatch = warnings.iter().any(|w| {
        matches!(
            w,
            rlf::LoadWarning::ParameterCountMismatch {
                name,
                source_count: 1,
                translation_count: 2,
                ..
            } if name == "energy"
        )
    });
    assert!(
        has_mismatch,
        "validation should detect parameter count mismatch for 'energy', got warnings: {warnings:?}"
    );
}

/// Seeded test: a translation that uses invalid tags for a language with
/// known valid tags triggers an `InvalidTag` warning.
/// Uses German (de) which has known valid tags: :masc, :fem, :neut.
#[test]
fn seeded_invalid_tag_detected() {
    strings::strings::register_source_phrases();
    let warnings = rlf::with_locale_mut(|locale| {
        locale
            .load_translations_str(
                "de",
                r#"
                    card = :bogus_invalid { one: "Karte", other: "Karten" };
                "#,
            )
            .expect("loading German with invalid tag should succeed");
        locale.validate_translations(SOURCE_LANGUAGE, "de")
    });
    let has_invalid_tag = warnings.iter().any(|w| {
        matches!(w, rlf::LoadWarning::InvalidTag { tag, language, .. } if tag == "bogus_invalid" && language == "de")
    });
    assert!(
        has_invalid_tag,
        "validation should detect invalid tag ':bogus_invalid' for German, got warnings: {warnings:?}"
    );
}

/// Seeded test: a translation that uses invalid variant keys for a language
/// with known valid variant keys triggers an `InvalidVariantKey` warning.
/// Uses Russian (ru) which has known valid variant keys.
#[test]
fn seeded_invalid_variant_key_detected() {
    strings::strings::register_source_phrases();
    let warnings = rlf::with_locale_mut(|locale| {
        locale
            .load_translations_str(
                "ru",
                r#"
                    card = { bogus_case: "карта", *other: "карты" };
                "#,
            )
            .expect("loading Russian with invalid variant key should succeed");
        locale.validate_translations(SOURCE_LANGUAGE, "ru")
    });
    let has_invalid_key = warnings.iter().any(|w| {
        matches!(
            w,
            rlf::LoadWarning::InvalidVariantKey { key, language, .. }
            if key == "bogus_case" && language == "ru"
        )
    });
    assert!(
        has_invalid_key,
        "validation should detect invalid variant key 'bogus_case' for Russian, got warnings: {warnings:?}"
    );
}

/// Seeded test: multiple validation errors in a single locale are all
/// reported, not just the first one.
#[test]
fn seeded_multiple_errors_all_reported() {
    strings::strings::register_source_phrases();
    let warnings = rlf::with_locale_mut(|locale| {
        locale
            .load_translations_str(
                "en-x-bad-multi",
                r#"
                    orphan_one = "does not exist in source";
                    orphan_two = "also does not exist";
                    energy($e, $extra) = "wrong param count {$e} {$extra}";
                "#,
            )
            .expect("loading seeded multi-error locale should succeed");
        locale.validate_translations(SOURCE_LANGUAGE, "en-x-bad-multi")
    });
    let unknown_count =
        warnings.iter().filter(|w| matches!(w, rlf::LoadWarning::UnknownPhrase { .. })).count();
    let mismatch_count = warnings
        .iter()
        .filter(|w| matches!(w, rlf::LoadWarning::ParameterCountMismatch { .. }))
        .count();
    assert!(
        unknown_count >= 2,
        "should detect at least 2 unknown phrases, found {unknown_count}; warnings: {warnings:?}"
    );
    assert!(
        mismatch_count >= 1,
        "should detect at least 1 parameter count mismatch, found {mismatch_count}; warnings: {warnings:?}"
    );
}

/// Seeded test: validation against a locale with zero phrases loaded
/// returns no errors (empty registries are handled gracefully).
#[test]
fn validation_with_unloaded_locale_returns_empty() {
    strings::strings::register_source_phrases();
    let warnings =
        rlf::with_locale(|locale| locale.validate_translations(SOURCE_LANGUAGE, "zz-nonexistent"));
    assert!(
        warnings.is_empty(),
        "validation against unloaded locale should return empty, got: {warnings:?}"
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
/// variant that does not exist on its parameter. This simulates the
/// "missing variant coverage" check from Appendix E.
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
/// tag that the phrase does not carry. This simulates the "missing tag
/// coverage" check from Appendix E.
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
/// successfully, proving the inverse of the failure case.
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
/// successfully, proving the inverse of the failure case.
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

/// End-to-end validation: the bracket locale, when loaded and validated,
/// produces zero warnings and has complete coverage of all source phrases.
#[test]
fn translation_validation_gate_integration() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");

    let warnings =
        rlf::with_locale(|locale| locale.validate_translations(SOURCE_LANGUAGE, BRACKET_LANGUAGE));
    assert!(
        warnings.is_empty(),
        "translation validation gate: bracket locale has {} warning(s):\n{}",
        warnings.len(),
        warnings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
    );

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
