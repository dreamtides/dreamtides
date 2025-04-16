use serde::{Deserialize, Serialize};

use crate::identifiers::CardId;

/// Describes the source of some mutation or query.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EffectSource {
    /// Mutation or query caused by the rules of the game, e.g. drawing a card
    /// for turn during a battle.
    Game,

    /// Mutation or query caused by a card, e.g. when a character is banished by
    /// an event card.
    Card(CardId),
}
