use core_data::numerics::{Energy, Points};

/// Represents the state of a player within a battle.
///
/// Terminology note: we always refer to a participant in a battle as a
/// "player". We use the term "user" to refer to the human playing the game. For
/// information about the user's overall save file state, refer to `UserData`.
#[derive(Clone, Debug)]
pub struct PlayerData {
    pub points: Points,
    pub current_energy: Energy,
    pub produced_energy: Energy,
}
