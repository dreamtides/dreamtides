use core_data::numerics::TurnId;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::CharacterId;

/// Identifies a turn within the game.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TurnData {
    /// Player whose turn it is or was.
    pub active_player: PlayerName,

    /// Identifies the turn.
    ///
    /// Each player's turn gets its own ID, so the first turn of the game is
    /// turn 0 for the starting player and then turn 1 for the next player.
    pub turn_id: TurnId,

    /// Current column position being resolved during the Judgment phase (0-7).
    pub judgment_position: u8,

    /// Characters that have been repositioned this turn, used to prevent
    /// infinite back-and-forth movement by the AI.
    pub moved_this_turn: Vec<CharacterId>,

    /// Characters that participated in a judgment (spark comparison) during
    /// the current Judgment phase. Each entry is (player, character_id,
    /// column). After all columns resolve, surviving participants return to
    /// back rank.
    pub judgment_participants: Vec<(PlayerName, CharacterId, u8)>,
}

impl Default for TurnData {
    fn default() -> Self {
        TurnData {
            active_player: PlayerName::One,
            turn_id: TurnId::default(),
            judgment_position: 0,
            moved_this_turn: Vec::new(),
            judgment_participants: Vec::new(),
        }
    }
}
