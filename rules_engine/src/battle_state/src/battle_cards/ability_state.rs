use std::collections::BTreeSet;

use crate::battle::card_id::{CardId, VoidCardId};
use crate::battle_cards::battle_card_state::ObjectId;
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

    /// State for abilities which persist until the end of the current turn.
    pub until_end_of_turn: UntilEndOfTurn,
}

/// Stores state for abilities which persist until the end of the current turn.
///
/// This struct is automatically dropped by the rules engine when a new turn
/// begins.
#[derive(Debug, Clone, Default)]
pub struct UntilEndOfTurn {
    /// Cards which should be prevented from being dissolved this turn.
    pub prevent_dissolved: BTreeSet<ObjectId>,
}
