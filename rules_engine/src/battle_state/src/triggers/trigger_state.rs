use crate::battle::card_id::CardId;
use crate::core::effect_source::EffectSource;
use crate::triggers::trigger::Trigger;
use crate::triggers::trigger_listeners::TriggerListeners;

/// Stores state for the trigger system.
///
/// This struct keeps track of 1) cards currently listening for a trigger and 2)
/// triggers which have fired. Each time a battle action finishes resolving,
/// *IF* there are currently no active player prompts, all triggers recorded are
/// resolved in the order in which they were recorded. Order of listeners being
/// invoked within a single event is arbitrary (currently in CardID order).
#[derive(Debug, Clone, Default)]
pub struct TriggerState {
    pub listeners: TriggerListeners,
    pub events: Vec<TriggerForListener>,
}

/// A record of a trigger event for a specific listener.
#[derive(Debug, Clone)]
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
                self.events.push(TriggerForListener { source, listener, trigger });
            }
        }
    }
}
