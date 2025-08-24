use core_data::numerics::TurnId;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

/// Identifies a turn within the game.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TurnData {
    /// Player whose turn it is or was.
    pub active_player: PlayerName,

    /// Identifies the turn.
    ///
    /// Each player's turn gets its own ID, so the first turn of the game is
    /// turn 0 for the starting player and then turn 1 for the next player.
    pub turn_id: TurnId,
}

impl Default for TurnData {
    fn default() -> Self {
        TurnData { active_player: PlayerName::One, turn_id: TurnId::default() }
    }
}
