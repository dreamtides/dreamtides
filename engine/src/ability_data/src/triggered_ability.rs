use serde::{Deserialize, Serialize};

use crate::effect::Effect;
use crate::trigger_event::TriggerEvent;

/// A triggered ability is an effect which happens when some triggering
/// event occurs, typically while its card is in play. Indicated in card
/// text by "When", "Whenever", "At", or by a trigger keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub trigger: TriggerEvent,
    pub effect: Effect,
}

impl TriggeredAbility {
    pub fn new(trigger: TriggerEvent, effect: Effect) -> Self {
        Self { trigger, effect }
    }
}
