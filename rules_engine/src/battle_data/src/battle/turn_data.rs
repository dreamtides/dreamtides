use core_data::numerics::TurnNumber;
use core_data::types::PlayerName;

/// Identifies a turn within the game.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TurnData {
    /// Player whose turn it is or was.
    pub active_player: PlayerName,

    /// Turn number for that player.
    ///
    /// The first turn of the game is turn 0.
    pub turn_number: TurnNumber,
}
