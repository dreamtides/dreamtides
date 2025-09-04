use serde::{Deserialize, Serialize};

use crate::battle::card_id::{CardId, CharacterId};
use crate::battle_cards::battle_card_state::CardObjectId;
use crate::battle_cards::card_set::CardSet;
use crate::battle_cards::dreamwell_data::BattleDreamwellCardId;

/// Stores state for abilities of a player in this battle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbilityState {
    /// Cards which should be banished when they are moved to any zone other
    /// than the stack or battlefield.
    #[serde(default)]
    pub banish_when_leaves_play: CardSet<CardId>,

    /// State for abilities which persist until the end of the current turn.
    #[serde(default)]
    pub until_end_of_turn: UntilEndOfTurn,
}

/// Stores state for abilities which persist until the end of the current turn.
///
/// This struct is automatically dropped by the rules engine when a new turn
/// begins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UntilEndOfTurn {
    /// The dreamwell card for the current turn, if any.
    ///
    /// Selected during the 'dreamwell' phase of the turn.
    pub active_dreamwell_card: Option<BattleDreamwellCardId>,

    /// Characters which should be prevented from being dissolved this turn.
    pub prevent_dissolved: Vec<CardObjectId<CharacterId>>,
}
