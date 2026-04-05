use serde::{Deserialize, Serialize};

use crate::battle::card_id::CharacterId;
use crate::battle_cards::card_set::CardSet;

/// Represents the battlefield with front and back ranks for a single player.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Battlefield {
    pub front: [Option<CharacterId>; 8],
    pub back: [Option<CharacterId>; 8],
}

impl Battlefield {
    /// Returns the total number of characters across both ranks.
    pub fn character_count(&self) -> usize {
        self.front.iter().filter(|s| s.is_some()).count()
            + self.back.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if both ranks are full (16 total characters).
    pub fn is_full(&self) -> bool {
        self.character_count() == 16
    }

    /// Returns the index of the first empty slot in the back rank, if any.
    pub fn first_empty_back_slot(&self) -> Option<usize> {
        self.back.iter().position(Option::is_none)
    }

    /// Returns the index of the first empty slot in the front rank, if any.
    pub fn first_empty_front_slot(&self) -> Option<usize> {
        self.front.iter().position(Option::is_none)
    }

    /// Returns true if the given character is on the battlefield.
    pub fn contains(&self, id: CharacterId) -> bool {
        self.front.contains(&Some(id)) || self.back.contains(&Some(id))
    }

    /// Removes a character from whichever rank it occupies.
    ///
    /// Returns true if the character was found and removed.
    pub fn remove(&mut self, id: CharacterId) -> bool {
        for slot in &mut self.front {
            if *slot == Some(id) {
                *slot = None;
                return true;
            }
        }
        for slot in &mut self.back {
            if *slot == Some(id) {
                *slot = None;
                return true;
            }
        }
        false
    }

    /// Adds a character to the back rank, returning the index where placed.
    ///
    /// Panics if the back rank is full.
    pub fn add_to_back_rank(&mut self, id: CharacterId) -> usize {
        let index =
            self.first_empty_back_slot().expect("Cannot add to back rank: back rank is full");
        self.back[index] = Some(id);
        index
    }

    /// Returns true if the character is in the front rank.
    pub fn is_in_front_rank(&self, id: CharacterId) -> bool {
        self.front.contains(&Some(id))
    }

    /// Returns true if the character is in the back rank.
    pub fn is_in_back_rank(&self, id: CharacterId) -> bool {
        self.back.contains(&Some(id))
    }

    /// Returns all characters on the battlefield.
    pub fn all_characters(&self) -> Vec<CharacterId> {
        self.front.iter().chain(self.back.iter()).filter_map(|s| *s).collect()
    }

    /// Returns true if the battlefield has no characters.
    pub fn is_empty(&self) -> bool {
        self.character_count() == 0
    }

    /// Returns a [CardSet] containing all characters on the battlefield.
    pub fn as_card_set(&self) -> CardSet<CharacterId> {
        self.all_characters().into_iter().collect()
    }
}
