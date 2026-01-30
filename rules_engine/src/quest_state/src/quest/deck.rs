use core_data::identifiers::BaseCardId;
use serde::{Deserialize, Serialize};
use tabula_data::card_definition::CardDefinition;
use tabula_data::tabula::Tabula;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuestDeckCardId(usize);

/// Represents a player's deck during a quest or battle.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Deck {
    pub cards: Vec<CardDefinition>,
}

impl Deck {
    /// Returns an iterator over the cards in the deck, paired with their
    /// [QuestDeckCardId].
    pub fn cards(&self) -> impl Iterator<Item = (QuestDeckCardId, &CardDefinition)> {
        self.cards.iter().enumerate().map(|(i, c)| (QuestDeckCardId(i), c))
    }

    /// Returns the card with the given [QuestDeckCardId].
    pub fn get_card(&self, id: QuestDeckCardId) -> &CardDefinition {
        &self.cards[id.0]
    }

    /// Inserts `count` copies of the card with the given [BaseCardId] into the
    /// deck.
    pub fn insert_copies(&mut self, tabula: &Tabula, id: BaseCardId, count: usize) {
        let card = tabula
            .cards
            .get(&id)
            .unwrap_or_else(|| panic!("Card definition not found for card id: {id:?}"));
        for _ in 0..count {
            self.cards.push(card.clone());
        }
    }

    pub fn push_card_and_get_id(&mut self, card: CardDefinition) -> QuestDeckCardId {
        let id = QuestDeckCardId(self.cards.len());
        self.cards.push(card);
        id
    }
}
