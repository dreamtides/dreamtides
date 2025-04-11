use serde::{Deserialize, Serialize};

/// Describes the source of some mutation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Mutation caused by the rules of the game, e.g. drawing a card for turn
    /// during a battle.
    Game,
}
