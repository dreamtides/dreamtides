use core_data::numerics::Energy;
use core_data::types::PlayerName;
use enumset::EnumSetType;

use crate::battle::card_id::{CharacterId, StackCardId, VoidCardId};

/// Represents an event which can occur during a battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Abandonded(VoidCardId),
    Banished(VoidCardId),
    Discarded(VoidCardId),
    Dissolved(VoidCardId),
    DrewAllCardsInCopyOfDeck(PlayerName),
    EndOfTurn(PlayerName),
    GainedEnergy(PlayerName, Energy),
    Judgment(PlayerName),
    Materialized(CharacterId),
    PlayedCardFromHand(StackCardId),
}

#[derive(EnumSetType, Debug)]
pub enum TriggerName {
    Abandonded,
    Banished,
    Discarded,
    Dissolved,
    DrewAllCardsInCopyOfDeck,
    EndOfTurn,
    GainedEnergy,
    Judgment,
    Materialized,
    PlayedCardFromHand,
}

impl Trigger {
    /// Returns the [TriggerName] variant of this trigger.
    pub fn name(&self) -> TriggerName {
        match self {
            Trigger::Abandonded(..) => TriggerName::Abandonded,
            Trigger::Banished(..) => TriggerName::Banished,
            Trigger::Discarded(..) => TriggerName::Discarded,
            Trigger::Dissolved(..) => TriggerName::Dissolved,
            Trigger::DrewAllCardsInCopyOfDeck(..) => TriggerName::DrewAllCardsInCopyOfDeck,
            Trigger::EndOfTurn(..) => TriggerName::EndOfTurn,
            Trigger::GainedEnergy(..) => TriggerName::GainedEnergy,
            Trigger::Judgment(..) => TriggerName::Judgment,
            Trigger::Materialized(..) => TriggerName::Materialized,
            Trigger::PlayedCardFromHand(..) => TriggerName::PlayedCardFromHand,
        }
    }
}
