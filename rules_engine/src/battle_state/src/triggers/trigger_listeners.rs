use crate::battle::card_id::CardId;
use crate::battle_cards::card_set::CardSet;
use crate::triggers::trigger::Trigger;

/// Tracks cards which are currently listening for a trigger.
#[derive(Debug, Clone, Default)]
pub struct TriggerListeners {
    pub played_card_from_hand: CardSet<CardId>,
}

impl TriggerListeners {
    pub fn listeners(&self, trigger: Trigger) -> &CardSet<CardId> {
        match trigger {
            Trigger::PlayedCardFromHand(..) => &self.played_card_from_hand,
        }
    }
}
