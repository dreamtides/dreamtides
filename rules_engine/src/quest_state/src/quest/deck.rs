use std::collections::BTreeMap;

use core_data::identifiers::CardIdentity;
use tabula_data::card_definition::CardDefinition;
use tabula_data::tabula::Tabula;

/// Represents a player's deck during a quest or battle.
#[derive(Clone, Debug)]
pub struct Deck {
    /// Cards in the deck and their counts.
    pub cards: BTreeMap<CardIdentity, usize>,
}

/// Returns the [CardDefinition] for a given [CardIdentity].
///
/// Panics if the card definition is not found.
pub fn card_definition(tabula: &Tabula, identity: CardIdentity) -> &CardDefinition {
    tabula
        .test_cards
        .get(&identity)
        .unwrap_or_else(|| panic!("Card definition not found for card identity: {identity:?}"))
}
