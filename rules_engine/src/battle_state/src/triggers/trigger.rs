use enumset::EnumSetType;

use crate::battle::card_id::StackCardId;

/// Represents an event which can occur during a battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    PlayedCardFromHand(StackCardId),
}

#[derive(EnumSetType, Debug)]
pub enum TriggerName {
    PlayedCardFromHand,
}

impl Trigger {
    /// Returns the [TriggerName] variant of this trigger.
    pub fn name(&self) -> TriggerName {
        match self {
            Trigger::PlayedCardFromHand(..) => TriggerName::PlayedCardFromHand,
        }
    }
}
