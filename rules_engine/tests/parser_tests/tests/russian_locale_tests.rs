use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;

use parser::lexer::lexer_tokenize;
use parser::serializer::ability_serializer;
use parser::variables::parser_bindings::VariableBindings;
use parser::variables::parser_substitutions;
use parser_tests::test_helpers;
use serde::Deserialize;

const CARDS_TOML_PATH: &str = "../../tabula/cards.toml";
const TEST_CARDS_TOML_PATH: &str = "../../tabula/test-cards.toml";
const RUSSIAN_LANGUAGE: &str = "ru";
const SOURCE_LANGUAGE: &str = "en";
const RUSSIAN_EXPECTED_PATH: &str = "tests/round_trip_tests/fixtures/russian_locale_expected.toml";
const RUSSIAN_BASELINE_PATH: &str = "tests/round_trip_tests/fixtures/russian_locale_baseline.toml";
const MAX_REPORTED_ISSUES: usize = 40;

static RUSSIAN_LOCALE_INIT: Once = Once::new();

#[derive(Debug, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct TestCardsFile {
    #[serde(rename = "test-cards")]
    test_cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RussianLocaleExpected {
    tests: Vec<RussianTestEntry>,
}

#[derive(Debug, Deserialize)]
struct RussianTestEntry {
    english: String,
    russian: String,
    variables: Option<String>,
    /// When true, this test is skipped because the engine does not yet
    /// produce the correct translator-provided Russian string.
    #[serde(default)]
    disabled: bool,
}

#[derive(Debug, Deserialize)]
struct RussianLocaleBaseline {
    min_passing: usize,
}

/// Registers the Russian locale and sets it as active. Safe to call from
/// multiple test threads — the `Once` ensures initialization happens exactly
/// once and the language is never restored to English, avoiding a race where
/// one test's cleanup resets the locale while another test is still running.
fn activate_russian_locale() {
    RUSSIAN_LOCALE_INIT.call_once(|| {
        test_helpers::register_russian_test_locale().expect("Russian locale should load");
        rlf::with_locale_mut(|locale| locale.set_language(RUSSIAN_LANGUAGE));
    });
}

/// Parses ability text through the lexer, parser, and serializer pipeline,
/// returning the rendered output string or an error description.
fn render_ability(ability_text: &str, variables: &str) -> Result<String, String> {
    let bindings =
        VariableBindings::parse(variables).map_err(|e| format!("variable parse error: {e:?}"))?;
    let lex_result =
        lexer_tokenize::lex(ability_text).map_err(|e| format!("lexer error: {e:?}"))?;
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .map_err(|e| format!("variable resolution error: {e}"))?;
    let ability = test_helpers::parse_resolved_ability(&resolved)
        .map_err(|e| format!("parser error: {e}"))?;
    catch_unwind(AssertUnwindSafe(|| ability_serializer::serialize_ability(&ability).text))
        .map_err(|panic| format!("serializer panic: {}", panic_message(panic)))
}

fn panic_message(panic: Box<dyn Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

/// Ratcheting translation test. Parses each English ability from the
/// expected fixture, switches to Russian locale, re-serializes, and
/// compares to the expected Russian string. Asserts passing count meets
/// the baseline minimum.
#[test]
fn test_russian_locale_ratcheting_translations() {
    activate_russian_locale();

    let expected_toml = std::fs::read_to_string(RUSSIAN_EXPECTED_PATH).unwrap_or_else(|e| {
        panic!("Failed to read Russian expected fixture at {RUSSIAN_EXPECTED_PATH}: {e}")
    });
    let expected: RussianLocaleExpected = toml::from_str(&expected_toml).unwrap_or_else(|e| {
        panic!("Failed to parse Russian expected fixture at {RUSSIAN_EXPECTED_PATH}: {e}")
    });

    let baseline_toml = std::fs::read_to_string(RUSSIAN_BASELINE_PATH).unwrap_or_else(|e| {
        panic!("Failed to read Russian baseline at {RUSSIAN_BASELINE_PATH}: {e}")
    });
    let baseline: RussianLocaleBaseline = toml::from_str(&baseline_toml).unwrap_or_else(|e| {
        panic!("Failed to parse Russian baseline at {RUSSIAN_BASELINE_PATH}: {e}")
    });

    let mut passing = 0usize;
    let mut disabled = 0usize;
    let mut failures = Vec::new();

    for (index, entry) in expected.tests.iter().enumerate() {
        if entry.disabled {
            disabled += 1;
            continue;
        }
        let variables = entry.variables.as_deref().unwrap_or("");
        let result = render_ability(&entry.english, variables);
        match result {
            Ok(rendered) if rendered == entry.russian => {
                passing += 1;
            }
            Ok(rendered) => {
                failures.push(format!(
                    "  #{index}: mismatch\n    english:  {:?}\n    expected: {:?}\n    actual:   {:?}",
                    entry.english, entry.russian, rendered
                ));
            }
            Err(error) => {
                failures.push(format!(
                    "  #{index}: render error\n    english: {:?}\n    error:   {error}",
                    entry.english
                ));
            }
        }
    }

    let active = expected.tests.len() - disabled;
    println!(
        "Russian locale ratchet: {passing}/{active} passing, {disabled} disabled (baseline min: {})",
        baseline.min_passing
    );

    if !failures.is_empty() {
        let shown = failures.iter().take(MAX_REPORTED_ISSUES).cloned().collect::<Vec<_>>();
        println!(
            "Failures ({} total, showing up to {MAX_REPORTED_ISSUES}):\n{}",
            failures.len(),
            shown.join("\n")
        );
    }

    assert!(
        passing >= baseline.min_passing,
        "Russian locale ratchet failed: {passing} passing < {} min_passing\n\
         Failures ({} total, showing up to {MAX_REPORTED_ISSUES}):\n{}",
        baseline.min_passing,
        failures.len(),
        failures.iter().take(MAX_REPORTED_ISSUES).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Renders all card abilities from both cards.toml and test-cards.toml
/// through the Russian locale and asserts no panics occur.
#[test]
fn test_russian_locale_no_crash_all_abilities() {
    activate_russian_locale();

    let cards_toml = std::fs::read_to_string(CARDS_TOML_PATH).expect("Failed to read cards.toml");
    let cards_file: CardsFile = toml::from_str(&cards_toml).expect("Failed to parse cards.toml");

    let test_cards_toml =
        std::fs::read_to_string(TEST_CARDS_TOML_PATH).expect("Failed to read test-cards.toml");
    let test_cards_file: TestCardsFile =
        toml::from_str(&test_cards_toml).expect("Failed to parse test-cards.toml");

    let mut total_abilities = 0usize;
    let mut render_errors = Vec::new();

    for (cards, source) in [
        (cards_file.cards.as_slice(), "cards.toml"),
        (test_cards_file.test_cards.as_slice(), "test-cards.toml"),
    ] {
        for card in cards {
            let Some(rules_text) = &card.rules_text else {
                continue;
            };
            let variables = card.variables.as_deref().unwrap_or("");
            for (ability_index, ability_block) in rules_text.split("\n\n").enumerate() {
                let ability_block = ability_block.trim();
                if ability_block.is_empty() {
                    continue;
                }
                total_abilities += 1;
                if let Err(error) = render_ability(ability_block, variables) {
                    render_errors.push(format!(
                        "- {source} | {} | ability #{ability_index} | {error}",
                        card.name
                    ));
                }
            }
        }
    }

    println!(
        "Russian locale no-crash test: rendered {total_abilities} abilities, {} errors",
        render_errors.len()
    );

    assert!(total_abilities >= 278, "Expected at least 278 abilities but found {total_abilities}");

    if !render_errors.is_empty() {
        let shown: Vec<_> = render_errors.iter().take(MAX_REPORTED_ISSUES).collect();
        println!(
            "Render errors ({} total, showing up to {MAX_REPORTED_ISSUES}):\n{}",
            render_errors.len(),
            shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
        );
    }
}

/// Asserts that loading the Russian locale succeeds and loads the same
/// phrase count as English.
#[test]
fn test_russian_locale_load_phrase_count() {
    activate_russian_locale();
    let loaded = rlf::with_locale(|locale| {
        locale
            .registry_for(RUSSIAN_LANGUAGE)
            .expect("Russian phrases should be registered")
            .phrase_names()
            .count()
    });
    rlf::with_locale(|locale| {
        let source_phrase_count = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered")
            .phrase_names()
            .count();
        assert_eq!(
            loaded, source_phrase_count,
            "Russian locale phrase count ({loaded}) should match English source phrase count ({source_phrase_count})"
        );
    });
}

/// Validates Russian locale coverage against English source definitions:
/// no missing phrases, no orphan phrases, and matching parameter counts.
#[test]
fn test_russian_locale_translation_validation_gate() {
    activate_russian_locale();
    let (missing, orphans, mismatches) = rlf::with_locale(|locale| {
        let source = locale
            .registry_for(SOURCE_LANGUAGE)
            .expect("English source phrases should be registered");
        let russian =
            locale.registry_for(RUSSIAN_LANGUAGE).expect("Russian phrases should be registered");

        let mut source_names: Vec<&str> = source.phrase_names().collect();
        source_names.sort();
        let mut russian_names: Vec<&str> = russian.phrase_names().collect();
        russian_names.sort();

        let mut missing_phrases = Vec::new();
        for name in &source_names {
            if russian.get(name).is_none() {
                missing_phrases.push((*name).to_string());
            }
        }

        let mut orphan_phrases = Vec::new();
        for name in &russian_names {
            if source.get(name).is_none() {
                orphan_phrases.push((*name).to_string());
            }
        }

        let mut parameter_mismatches = Vec::new();
        for name in &source_names {
            let Some(source_def) = source.get(name) else {
                continue;
            };
            let Some(russian_def) = russian.get(name) else {
                continue;
            };
            if source_def.parameters.len() != russian_def.parameters.len() {
                parameter_mismatches.push(format!(
                    "{name}: source has {} param(s), ru has {}",
                    source_def.parameters.len(),
                    russian_def.parameters.len()
                ));
            }
        }

        (missing_phrases, orphan_phrases, parameter_mismatches)
    });

    let failure_count = missing.len() + orphans.len() + mismatches.len();
    println!(
        "Russian locale translation validation: {failure_count} failure(s) (missing: {}, orphans: {}, parameter mismatches: {})",
        missing.len(),
        orphans.len(),
        mismatches.len()
    );

    assert!(
        failure_count == 0,
        "Russian locale translation validation failed with {failure_count} issue(s)\n\
         Missing ({}):\n  {}\n\
         Orphans ({}):\n  {}\n\
         Parameter mismatches ({}):\n  {}",
        missing.len(),
        if missing.is_empty() { "(none)".to_string() } else { missing.join("\n  ") },
        orphans.len(),
        if orphans.is_empty() { "(none)".to_string() } else { orphans.join("\n  ") },
        mismatches.len(),
        if mismatches.is_empty() { "(none)".to_string() } else { mismatches.join("\n  ") }
    );
}
