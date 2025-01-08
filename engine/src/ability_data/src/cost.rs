use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::predicate::Predicate;

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cost {
    NoCost,
    Energy(Energy),
    BanishCardsFromEnemyVoid(u64),
    BanishCardsFromYourVoid(u64),
    BanishAllCardsFromYourVoid,
    BanishFromHand(Predicate),
    AbandonCharacters(Predicate, u64),
    DiscardHand,
}
