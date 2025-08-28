use std::sync::Arc;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tabula_data::card_definitions::dreamwell_card_definition::DreamwellCardDefinition;

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
