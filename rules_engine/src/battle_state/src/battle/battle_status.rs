use core_data::types::PlayerName;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BattleStatus {
    /// Initial step of battle setup.
    Setup,

    /// Players resolve mulligans in sequence.
    ResolveMulligans,

    /// Battle is currently ongoing
    Playing,

    /// Battle has ended and the [PlayerName] player has won.
    ///
    /// If the winner is None, the battle has ended in a draw.
    GameOver { winner: Option<PlayerName> },
}
