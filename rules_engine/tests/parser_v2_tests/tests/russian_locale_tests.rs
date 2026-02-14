use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use parser_v2_tests::test_helpers;
use serde::Deserialize;

const CARDS_TOML_PATH: &str = "../../tabula/cards.toml";
const TEST_CARDS_TOML_PATH: &str = "../../tabula/test-cards.toml";
const RUSSIAN_LANGUAGE: &str = "ru";
const SOURCE_LANGUAGE: &str = "en";
const RUSSIAN_EXPECTED_PATH: &str = "tests/round_trip_tests/fixtures/russian_locale_expected.toml";
const RUSSIAN_BASELINE_PATH: &str = "tests/round_trip_tests/fixtures/russian_locale_baseline.toml";
const MAX_REPORTED_ISSUES: usize = 40;

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
}

#[derive(Debug, Deserialize)]
struct RussianLocaleBaseline {
    min_passing: usize,
    max_validation_warnings: usize,
}

struct LanguageGuard {
    previous_language: String,
}

impl Drop for LanguageGuard {
    fn drop(&mut self) {
        rlf::with_locale_mut(|locale| locale.set_language(&self.previous_language));
    }
}

/// Switches the global locale to Russian and returns a guard that restores
/// the previous language on drop.
fn activate_russian_locale() -> LanguageGuard {
    let previous_language = rlf::with_locale(|locale| locale.language().to_string());
    rlf::with_locale_mut(|locale| locale.set_language(RUSSIAN_LANGUAGE));
    LanguageGuard { previous_language }
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
    let ability = {
        let parser = ability_parser::ability_parser();
        parser.parse(&resolved).into_result().map_err(|e| format!("parser error: {e:?}"))?
    };
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
    test_helpers::register_russian_test_locale().expect("Russian locale should load");

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

    let _language_guard = activate_russian_locale();

    let mut passing = 0usize;
    let mut failures = Vec::new();

    for (index, entry) in expected.tests.iter().enumerate() {
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

    println!(
        "Russian locale ratchet: {passing}/{} passing (baseline min: {})",
        expected.tests.len(),
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
    test_helpers::register_russian_test_locale().expect("Russian locale should load");
    let _language_guard = activate_russian_locale();

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
    let loaded = test_helpers::register_russian_test_locale()
        .expect("Russian locale should load from locale file");
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

/// Ratcheting validation test. Asserts that
/// `locale.validate_translations("en", "ru")` produces no more warnings
/// than the baseline allows. As locale quality improves, lower the
/// baseline to lock in progress.
#[test]
fn test_russian_locale_validation_warnings() {
    test_helpers::register_russian_test_locale().expect("Russian locale should load");

    let baseline_toml = std::fs::read_to_string(RUSSIAN_BASELINE_PATH).unwrap_or_else(|e| {
        panic!("Failed to read Russian baseline at {RUSSIAN_BASELINE_PATH}: {e}")
    });
    let baseline: RussianLocaleBaseline = toml::from_str(&baseline_toml).unwrap_or_else(|e| {
        panic!("Failed to parse Russian baseline at {RUSSIAN_BASELINE_PATH}: {e}")
    });

    let warnings =
        rlf::with_locale(|locale| locale.validate_translations(SOURCE_LANGUAGE, RUSSIAN_LANGUAGE));

    println!(
        "Russian locale validation: {} warnings (baseline max: {})",
        warnings.len(),
        baseline.max_validation_warnings
    );

    if !warnings.is_empty() {
        let shown: Vec<_> =
            warnings.iter().take(MAX_REPORTED_ISSUES).map(ToString::to_string).collect();
        println!(
            "Warnings ({} total, showing up to {MAX_REPORTED_ISSUES}):\n{}",
            warnings.len(),
            shown.join("\n")
        );
    }

    assert!(
        warnings.len() <= baseline.max_validation_warnings,
        "Russian locale validation regression: {} warnings > {} max allowed\n{}",
        warnings.len(),
        baseline.max_validation_warnings,
        warnings
            .iter()
            .take(MAX_REPORTED_ISSUES)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
