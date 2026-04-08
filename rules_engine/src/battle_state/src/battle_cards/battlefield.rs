use serde::{Deserialize, Serialize};

use crate::battle::card_id::CharacterId;
use crate::battle_cards::card_set::CardSet;

/// Represents the battlefield with front and back ranks for a single player.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Battlefield {
    pub front: [Option<CharacterId>; 4],
    pub back: [Option<CharacterId>; 5],
}

impl Battlefield {
    /// Returns the total number of characters across both ranks.
    pub fn character_count(&self) -> usize {
        self.front.iter().filter(|s| s.is_some()).count()
            + self.back.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if the battlefield has 9 or more characters (the maximum).
    pub fn is_full(&self) -> bool {
        self.character_count() >= 9
    }

    /// Returns true if all 5 back-row slots are occupied.
    pub fn back_row_is_full(&self) -> bool {
        self.back.iter().all(Option::is_some)
    }

    /// Returns the front-row slot indices that the given back-row slot
    /// supports in the staggered grid layout.
    ///
    /// B0→[F0], B1→[F0,F1], B2→[F1,F2], B3→[F2,F3], B4→[F3]
    pub fn supported_front_slots(back_slot: usize) -> &'static [usize] {
        match back_slot {
            0 => &[0],
            1 => &[0, 1],
            2 => &[1, 2],
            3 => &[2, 3],
            4 => &[3],
            _ => &[],
        }
    }

    /// Returns the back-row slot indices that support the given front-row
    /// slot in the staggered grid layout.
    ///
    /// F0→[B0,B1], F1→[B1,B2], F2→[B2,B3], F3→[B3,B4]
    pub fn supporting_back_slots(front_slot: usize) -> &'static [usize] {
        match front_slot {
            0 => &[0, 1],
            1 => &[1, 2],
            2 => &[2, 3],
            3 => &[3, 4],
            _ => &[],
        }
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

    /// Moves a character from the front rank to the back rank, preferring the
    /// same column index. If that back-rank slot is occupied, the character is
    /// placed in the first available back-rank slot instead.
    ///
    /// Returns false if the character is not in the front rank or no back-rank
    /// slot is available.
    pub fn return_to_back_rank(&mut self, id: CharacterId) -> bool {
        let Some(front_col) = self.front.iter().position(|s| *s == Some(id)) else {
            return false;
        };
        self.front[front_col] = None;
        if self.back[front_col].is_none() {
            self.back[front_col] = Some(id);
        } else if let Some(slot) = self.first_empty_back_slot() {
            self.back[slot] = Some(id);
        } else {
            self.front[front_col] = Some(id);
            return false;
        }
        true
    }

    /// Returns a [CardSet] containing all characters on the battlefield.
    pub fn as_card_set(&self) -> CardSet<CharacterId> {
        self.all_characters().into_iter().collect()
    }
}
