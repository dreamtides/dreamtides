use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::battle::card_id::CardId;
use crate::core::effect_source::EffectSource;
use crate::triggers::trigger::Trigger;
use crate::triggers::trigger_listeners::TriggerListeners;

/// Stores state for the trigger system.
///
/// This struct keeps track of 1) cards currently listening for a trigger and 2)
/// triggers which have fired. Each time a battle action finishes resolving,
/// *IF* there are currently no active player prompts, all triggers recorded are
/// resolved in the order in which they were recorded. Triggers are also fired
/// at the end of each player's turn. Triggers fire in first-in-first-out
/// (queue) order.
///
/// Order of listeners being invoked within a single event is arbitrary
/// (currently in CardID order).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerState {
    #[serde(default)]
    pub listeners: TriggerListeners,
    #[serde(default)]
    pub events: VecDeque<TriggerForListener>,
}

/// A record of a trigger event for a specific listener.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerForListener {
    pub source: EffectSource,
    pub listener: CardId,
    pub trigger: Trigger,
}

impl TriggerState {
    /// Records a new trigger event.
    ///
    /// For each card currently listening for this trigger, a
    /// [TriggerForListener] will be recorded.
    pub fn push(&mut self, source: EffectSource, trigger: Trigger) {
        if !self.listeners.listeners(trigger.name()).is_empty() {
            for listener in self.listeners.listeners(trigger.name()) {
                self.events.push_back(TriggerForListener { source, listener, trigger });
            }
        }
    }
}
