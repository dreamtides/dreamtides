use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::predicate::Predicate;

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cost {
    /// No cost required
    None,

    Energy(Energy),

    /// Banish N cards from your void.
    ///
    /// The owning card cannot be among those banished.
    BanishCardsFromYourVoid(u64),

    AbandonCharacter(Predicate),

    /// Discard all cards from your hand
    DiscardHand,
}
