use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::battle::card_id::{ActivatedAbilityId, CharacterId};
use crate::battle_cards::card_set::CardSet;

/// Stores state for activated abilities of a player in this battle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivatedAbilityState {
    /// Characters in play which have activated abilities.
    ///
    /// This is all characters with activated abilities, not only on the ones
    /// which can currently be activated, e.g. ability to pay the cost is not
    /// considered here.
    pub characters: CardSet<CharacterId>,

    /// Activated abilities which have been activated this turn cycle.
    ///
    /// Used for tracking once-per-turn abilities.
    pub activated_this_turn_cycle: BTreeSet<ActivatedAbilityId>,
}
