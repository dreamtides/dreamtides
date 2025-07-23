use core_data::identifiers::UserId;

/// Represents the overall state of a human user.
///
/// Terminology note: we always refer to the human playing the game as "the
/// user". We use the term "player" to refer to either of the two participants
/// in a battle, who may or may not be humans. Refer to the `PlayerData` struct
/// for data about a participant in a battle.
#[derive(Clone, Debug)]
pub struct UserData {
    pub id: UserId,
}
