use serde::{Deserialize, Serialize};

use crate::effect::Effect;
use crate::trigger_event::TriggerEvent;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggeredAbilityOptions {
    /// True if this ability can only trigger once per turn.
    pub once_per_turn: bool,

    /// True if this ability will last only until end of turn.
    pub until_end_of_turn: bool,
}

/// A triggered ability is an effect which happens when some triggering
/// event occurs, typically while its card is in play. Indicated in card
/// text by "When", "Whenever", "At", or by a trigger keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub trigger: TriggerEvent,
    pub effect: Effect,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<TriggeredAbilityOptions>,
}
