use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cost {
    /// No cost required
    None,

    Energy(Energy),

    /// Banish N cards from your void.
    BanishCardsFromYourVoid(u64),
}
