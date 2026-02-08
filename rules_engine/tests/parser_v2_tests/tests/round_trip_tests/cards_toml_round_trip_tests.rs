//! Round-trip tests for all cards in cards.toml.
//!
//! Verifies that parsing and serializing each card's rules text produces
//! rendered output matching the directly-rendered input text.

use parser_v2_tests::test_helpers;
use serde::Deserialize;

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

#[test]
fn test_all_cards_toml_round_trip() {
    let cards_toml =
        std::fs::read_to_string("../../tabula/cards.toml").expect("Failed to read cards.toml");
    let cards_file: CardsFile = toml::from_str(&cards_toml).expect("Failed to parse cards.toml");

    let mut errors = Vec::new();
    let mut success_count = 0;
    let mut total_abilities = 0;

    for card in &cards_file.cards {
        let Some(rules_text) = &card.rules_text else {
            continue;
        };

        let variables = card.variables.as_deref().unwrap_or("");

        for ability_block in rules_text.split("\n\n") {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            total_abilities += 1;

            match test_helpers::assert_rendered_match_for_toml(&card.name, ability_block, variables)
            {
                Ok(()) => success_count += 1,
                Err(error) => errors.push(error),
            }
        }
    }

    test_helpers::print_bulk_results("cards.toml", success_count, total_abilities, &errors);

    if !errors.is_empty() {
        panic!("\n{} abilities failed rendered comparison (see details above)", errors.len());
    }
}
