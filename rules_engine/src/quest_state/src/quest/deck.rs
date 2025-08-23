use core_data::identifiers::BaseCardId;
use tabula_data::card_definition::CardDefinition;
use tabula_data::tabula::Tabula;

/// Represents a player's deck during a quest or battle.
#[derive(Clone, Debug, Default)]
pub struct Deck {
    pub cards: Vec<CardDefinition>,
}

impl Deck {
    pub fn insert_copies(&mut self, tabula: &Tabula, id: BaseCardId, count: usize) {
        let card = tabula
            .test_cards
            .get(&id)
            .unwrap_or_else(|| panic!("Card definition not found for card id: {id:?}"));
        for _ in 0..count {
            self.cards.push(card.clone());
        }
    }
}
