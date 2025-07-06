use crate::battle::card_id::CardId;
use crate::battle_cards::card_set::CardSet;
use crate::triggers::trigger::TriggerName;

/// Tracks cards which are currently listening for a trigger.
#[derive(Debug, Clone, Default)]
pub struct TriggerListeners {
    pub played_card_from_hand: CardSet<CardId>,
}

impl TriggerListeners {
    /// Returns the set of cards currently listening for a trigger.
    pub fn listeners(&self, name: TriggerName) -> &CardSet<CardId> {
        match name {
            TriggerName::PlayedCardFromHand => &self.played_card_from_hand,
        }
    }

    /// Returns a mutable reference to the set of cards currently listening for
    /// a trigger.
    pub fn listeners_mut(&mut self, name: TriggerName) -> &mut CardSet<CardId> {
        match name {
            TriggerName::PlayedCardFromHand => &mut self.played_card_from_hand,
        }
    }

    /// Adds a card to the set of listeners for a trigger.
    pub fn add_listener(&mut self, name: TriggerName, card_id: CardId) {
        self.listeners_mut(name).insert(card_id);
    }

    /// Removes a card from the set of listeners for a trigger.
    pub fn remove_listener(&mut self, name: TriggerName, card_id: CardId) {
        self.listeners_mut(name).remove(card_id);
    }
}
