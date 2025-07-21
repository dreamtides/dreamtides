use crate::battle::card_id::{CardId, VoidCardId};
use crate::battle_cards::card_set::CardSet;
use crate::battle_player::player_map::PlayerMap;

/// Stores state for abilities of a player in this battle.
#[derive(Debug, Clone, Default)]
pub struct AbilityState {
    /// All cards currently in this player's void which have an ability which
    /// *may* let them be played from the void.
    pub has_play_from_void_ability: PlayerMap<CardSet<VoidCardId>>,

    /// Cards which should be banished when they are moved to any zone other
    /// than the stack or battlefield.
    pub banish_when_leaves_play: CardSet<CardId>,

    /// State for abilities of a player which persist until the end of their
    /// next judgment.
    pub player_turn: PlayerMap<PlayerTurnState>,
}

/// Stores state for abilities of a player which persist until the end of their
/// next judgment.
///
/// This struct is automatically dropped by the rules engine after each
/// judgement phase.
#[derive(Debug, Clone, Default)]
pub struct PlayerTurnState {
    /// Cards which should be prevented from being dissolved.
    pub prevent_dissolved: CardSet<CardId>,
}
