use core_data::numerics::{Energy, Points, Spark};

/// Represents the state of a player within a battle.
///
/// Terminology note: we always refer to a participant in a battle as a
/// "player". We use the term "user" to refer to the human playing the game. For
/// information about the user's overall save file state, refer to `UserData`.
#[derive(Clone, Debug, Default)]
pub struct PlayerData {
    /// Current score
    pub points: Points,

    /// Current energy
    pub current_energy: Energy,

    /// Energy produced each turn
    pub produced_energy: Energy,

    /// Additional spark for this player
    pub spark_bonus: Spark,
}
