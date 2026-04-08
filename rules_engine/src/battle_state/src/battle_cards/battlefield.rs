use serde::{Deserialize, Serialize};

use crate::battle::battle_rules_config::MAX_ROW_SIZE;
use crate::battle::card_id::CharacterId;
use crate::battle_cards::card_set::CardSet;

/// Represents the battlefield with front and back ranks for a single player.
///
/// Arrays are sized to [MAX_ROW_SIZE] but only the first `front_row_size` /
/// `back_row_size` slots (from `BattleRulesConfig`) are used.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Battlefield {
    pub front: [Option<CharacterId>; MAX_ROW_SIZE],
    pub back: [Option<CharacterId>; MAX_ROW_SIZE],
}

impl Battlefield {
    /// Returns the total number of characters across both ranks.
    pub fn character_count(&self) -> usize {
        self.front.iter().filter(|s| s.is_some()).count()
            + self.back.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if all back-row slots up to `size` are occupied.
    pub fn back_row_is_full(&self, size: usize) -> bool {
        self.back[..size].iter().all(Option::is_some)
    }

    /// Returns the front-row slot indices that the given back-row slot
    /// supports in the staggered grid layout.
    pub fn supported_front_slots(back_slot: usize, back_size: usize) -> Vec<usize> {
        if back_slot >= back_size {
            return vec![];
        }
        let mut result = Vec::new();
        if back_slot > 0 {
            result.push(back_slot - 1);
        }
        if back_slot < back_size - 1 {
            result.push(back_slot);
        }
        result
    }

    /// Returns the back-row slot indices that support the given front-row
    /// slot in the staggered grid layout.
    pub fn supporting_back_slots(front_slot: usize, front_size: usize) -> Vec<usize> {
        if front_slot >= front_size {
            return vec![];
        }
        vec![front_slot, front_slot + 1]
    }

    /// Returns the index of the first empty slot in the back rank.
    pub fn first_empty_back_slot(&self, size: usize) -> Option<usize> {
        self.back[..size].iter().position(Option::is_none)
    }

    /// Returns the index of the first empty slot in the front rank.
    pub fn first_empty_front_slot(&self, size: usize) -> Option<usize> {
        self.front[..size].iter().position(Option::is_none)
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
    pub fn add_to_back_rank(&mut self, id: CharacterId, size: usize) -> usize {
        let index =
            self.first_empty_back_slot(size).expect("Cannot add to back rank: back rank is full");
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
