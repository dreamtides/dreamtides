use crate::battle::card_id::StackCardId;

/// Represents an event which can occur during a battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    PlayedCardFromHand(StackCardId),
}
