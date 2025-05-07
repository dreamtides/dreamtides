use ai_data::game_ai::GameAI;
use core_data::identifiers::UserId;
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

/// Represents the state of a player within a battle.
///
/// Terminology note: we always refer to a participant in a battle as a
/// "player". We use the term "user" to refer to the human playing the game. For
/// information about the user's overall save file state, refer to `UserData`.
#[derive(Clone, Debug)]
pub struct PlayerData {
    /// Name of the player
    pub name: PlayerName,

    /// Contains the player's UserId or AI game agent info.
    pub player_type: PlayerType,

    /// Current score
    pub points: Points,

    /// Current energy
    pub current_energy: Energy,

    /// Energy produced each turn
    pub produced_energy: Energy,

    /// Additional spark for this player
    pub spark_bonus: Spark,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerType {
    User(UserId),
    Agent(GameAI),
}
