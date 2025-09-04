use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::battle::card_id::ActivatedAbilityId;

/// Stores state for activated abilities of a player in this battle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivatedAbilityState {
    /// Activated abilities which have been activated this turn cycle.
    ///
    /// Used for tracking once-per-turn abilities.
    #[serde(default)]
    pub activated_this_turn_cycle: BTreeSet<ActivatedAbilityId>,
}
