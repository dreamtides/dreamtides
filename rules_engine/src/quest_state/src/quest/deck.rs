use std::collections::BTreeMap;

use core_data::identifiers::CardName;

/// Represents a player's deck during a quest or battle.
#[derive(Clone, Debug)]
pub struct Deck {
    /// Cards in the deck and their counts.
    pub cards: BTreeMap<CardName, usize>,
}
