//! Parity regression test for the ability serializer.
//!
//! Verifies that phrase-based assembly produces fully-resolved output
//! (no unresolved RLF templates) for every ability in the full card corpus.
//! This confirms that serialized text requires no further RLF evaluation.

use std::collections::HashMap;

use parser_v2::lexer::lexer_tokenize;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use parser_v2_tests::test_helpers;
use serde::Deserialize;
use strings::strings;

#[derive(Debug, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TestCardsFile {
    #[serde(rename = "test-cards")]
    test_cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct DreamwellFile {
    dreamwell: Vec<Dreamwell>,
}

#[derive(Debug, Deserialize)]
struct Dreamwell {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TestDreamwellFile {
    #[serde(rename = "test-dreamwell")]
    test_dreamwell: Vec<Dreamwell>,
}

/// Disposition of a parity comparison result.
#[derive(Debug)]
enum Disposition {
    Match,
    Mismatch { serialized: String, resolved: String },
}

/// Result of a parity comparison for a single ability.
#[derive(Debug)]
struct ParityResult {
    card_name: String,
    ability_index: usize,
    disposition: Disposition,
}

/// Runs the parity comparison for a single ability.
///
/// Serializes the ability through the phrase-based serializer, then
/// independently runs RLF resolution on that same text. Compares the two.
fn check_parity(
    name: &str,
    ability_index: usize,
    ability_text: &str,
    variables: &str,
) -> Option<ParityResult> {
    let bindings = match VariableBindings::parse(variables) {
        Ok(b) => b,
        Err(_) => return None,
    };

    let lex_result = match lexer_tokenize::lex(ability_text) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let resolved = match parser_substitutions::resolve_variables(&lex_result.tokens, &bindings) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let ability = match test_helpers::parse_resolved_ability(&resolved) {
        Ok(a) => a,
        Err(_) => return None,
    };

    let serialized_text = ability_serializer::serialize_ability(&ability).text;

    strings::register_source_phrases();
    let resolved_text = rlf::with_locale(|locale| {
        locale.eval_str(&serialized_text, HashMap::new()).unwrap().to_string()
    });

    let disposition = if serialized_text == resolved_text {
        Disposition::Match
    } else {
        Disposition::Mismatch { serialized: serialized_text, resolved: resolved_text }
    };

    Some(ParityResult { card_name: name.to_string(), ability_index, disposition })
}

/// Collects parity results for all card-style entries.
fn collect_card_parity(cards_toml: &str, results: &mut Vec<ParityResult>) {
    let cards_file: CardsFile = toml::from_str(cards_toml).expect("Failed to parse cards TOML");
    for card in &cards_file.cards {
        let Some(rules_text) = &card.rules_text else { continue };
        let variables = card.variables.as_deref().unwrap_or("");
        for (i, block) in rules_text.split("\n\n").enumerate() {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            if let Some(result) = check_parity(&card.name, i, block, variables) {
                results.push(result);
            }
        }
    }
}

/// Collects parity results for all test-card-style entries.
fn collect_test_card_parity(test_cards_toml: &str, results: &mut Vec<ParityResult>) {
    let file: TestCardsFile =
        toml::from_str(test_cards_toml).expect("Failed to parse test-cards TOML");
    for card in &file.test_cards {
        let Some(rules_text) = &card.rules_text else { continue };
        let variables = card.variables.as_deref().unwrap_or("");
        for (i, block) in rules_text.split("\n\n").enumerate() {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            if let Some(result) = check_parity(&card.name, i, block, variables) {
                results.push(result);
            }
        }
    }
}

/// Collects parity results for all dreamwell-style entries.
fn collect_dreamwell_parity(dreamwell_toml: &str, results: &mut Vec<ParityResult>) {
    let file: DreamwellFile =
        toml::from_str(dreamwell_toml).expect("Failed to parse dreamwell TOML");
    for dw in &file.dreamwell {
        let Some(rules_text) = &dw.rules_text else { continue };
        let variables = dw.variables.as_deref().unwrap_or("");
        for (i, block) in rules_text.split("\n\n").enumerate() {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            if let Some(result) = check_parity(&dw.name, i, block, variables) {
                results.push(result);
            }
        }
    }
}

/// Collects parity results for all test-dreamwell entries.
fn collect_test_dreamwell_parity(test_dreamwell_toml: &str, results: &mut Vec<ParityResult>) {
    let file: TestDreamwellFile =
        toml::from_str(test_dreamwell_toml).expect("Failed to parse test-dreamwell TOML");
    for dw in &file.test_dreamwell {
        let Some(rules_text) = &dw.rules_text else { continue };
        let variables = dw.variables.as_deref().unwrap_or("");
        for (i, block) in rules_text.split("\n\n").enumerate() {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            if let Some(result) = check_parity(&dw.name, i, block, variables) {
                results.push(result);
            }
        }
    }
}

/// Regression test: verifies that the serializer's phrase-based assembly
/// produces fully-resolved output for every ability in the full corpus.
///
/// Compares `serialize_ability().text` against the result of running
/// `rlf::eval_str()` on that same text. Zero mismatches confirms that
/// serialized output contains no unresolved RLF templates.
#[test]
fn test_parity_serializer_output_fully_resolved() {
    let cards_toml =
        std::fs::read_to_string("../../tabula/cards.toml").expect("Failed to read cards.toml");
    let dreamwell_toml = std::fs::read_to_string("../../tabula/dreamwell.toml")
        .expect("Failed to read dreamwell.toml");
    let test_cards_toml = std::fs::read_to_string("../../tabula/test-cards.toml")
        .expect("Failed to read test-cards.toml");
    let test_dreamwell_toml = std::fs::read_to_string("../../tabula/test-dreamwell.toml")
        .expect("Failed to read test-dreamwell.toml");

    let mut results = Vec::new();
    collect_card_parity(&cards_toml, &mut results);
    collect_dreamwell_parity(&dreamwell_toml, &mut results);
    collect_test_card_parity(&test_cards_toml, &mut results);
    collect_test_dreamwell_parity(&test_dreamwell_toml, &mut results);

    let total = results.len();
    let mismatches: Vec<_> =
        results.iter().filter(|r| matches!(r.disposition, Disposition::Mismatch { .. })).collect();

    println!("\n========================================");
    println!("Parity Gate Results");
    println!("========================================");
    println!("Total abilities checked: {total}");
    println!("Matches: {}", total - mismatches.len());
    println!("Mismatches: {}", mismatches.len());
    println!("========================================\n");

    if !mismatches.is_empty() {
        for m in &mismatches {
            if let Disposition::Mismatch { serialized, resolved } = &m.disposition {
                eprintln!(
                    "MISMATCH: {}|{}|\n  serialized: {serialized:?}\n  resolved:   {resolved:?}",
                    m.card_name, m.ability_index
                );
            }
        }
        panic!(
            "{} parity mismatches found out of {total} abilities. \
             Serialized output must equal RLF-resolved output for all abilities.",
            mismatches.len()
        );
    }

    assert!(total > 0, "No abilities were checked; corpus may be empty or unreadable");
    println!("Parity regression PASSED: all {total} abilities produce fully-resolved output.");
}
