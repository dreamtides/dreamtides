use std::sync::Arc;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tabula_data::card_definitions::dreamwell_card_definition::DreamwellCardDefinition;
use tabula_data::tabula::Tabula;
use tabula_ids::card_lists::{self, DreamwellCardIdList};

/// The dreamwell is a deck of cards that is used during the dreamwell phase to
/// give players energy production and apply random effects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dreamwell {
    /// Cards in the dreamwell.
    #[serde(default)]
    cards: Arc<Vec<Arc<DreamwellCardDefinition>>>,

    /// Index of the next card to be drawn from the dreamwell.
    #[serde(default)]
    next_index: usize,
}

impl Dreamwell {
    /// Creates a new dreamwell from a [DreamwellCardIdList].
    pub fn from_card_list(tabula: &Tabula, list: DreamwellCardIdList) -> Self {
        let mut cards = Vec::new();
        for card_id in card_lists::dreamwell_card_id_list(list) {
            cards.push(Arc::new(
                tabula
                    .dreamwell_cards
                    .get(card_id)
                    .unwrap_or_else(|| panic!("Card {card_id:?} not found in tabula"))
                    .clone(),
            ));
        }
        Self { cards: Arc::new(cards), next_index: 0 }
    }

    /// Draws the next card from the dreamwell.
    ///
    /// Panics if the dreamwell is empty.
    pub fn draw_card(&mut self) -> Arc<DreamwellCardDefinition> {
        if self.next_index == 0 {
            // Randomly shuffle the dreamwell cards.
            let mut new_cards = self.cards.as_ref().clone();
            new_cards.shuffle(&mut rand::rng());
            self.cards = Arc::new(new_cards);
        }
        let card = self.cards[self.next_index].clone();
        self.next_index = (self.next_index + 1) % self.cards.len();
        card
    }
}
