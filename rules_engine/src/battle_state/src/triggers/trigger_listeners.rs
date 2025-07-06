use crate::battle::card_id::CardId;
use crate::battle_cards::card_set::CardSet;
use crate::triggers::trigger::TriggerName;

/// Tracks cards which are currently listening for a trigger.
#[derive(Debug, Clone, Default)]
pub struct TriggerListeners {
    pub abandoned: CardSet<CardId>,
    pub banished: CardSet<CardId>,
    pub discarded: CardSet<CardId>,
    pub dissolved: CardSet<CardId>,
    pub drew_all_cards_in_copy_of_deck: CardSet<CardId>,
    pub end_of_turn: CardSet<CardId>,
    pub gained_energy: CardSet<CardId>,
    pub judgment: CardSet<CardId>,
    pub materialized: CardSet<CardId>,
    pub played_card_from_hand: CardSet<CardId>,
}

impl TriggerListeners {
    /// Returns the set of cards currently listening for a trigger.
    pub fn listeners(&self, name: TriggerName) -> &CardSet<CardId> {
        match name {
            TriggerName::Abandonded => &self.abandoned,
            TriggerName::Banished => &self.banished,
            TriggerName::Discarded => &self.discarded,
            TriggerName::Dissolved => &self.dissolved,
            TriggerName::DrewAllCardsInCopyOfDeck => &self.drew_all_cards_in_copy_of_deck,
            TriggerName::EndOfTurn => &self.end_of_turn,
            TriggerName::GainedEnergy => &self.gained_energy,
            TriggerName::Judgment => &self.judgment,
            TriggerName::Materialized => &self.materialized,
            TriggerName::PlayedCardFromHand => &self.played_card_from_hand,
        }
    }

    /// Returns a mutable reference to the set of cards currently listening for
    /// a trigger.
    pub fn listeners_mut(&mut self, name: TriggerName) -> &mut CardSet<CardId> {
        match name {
            TriggerName::Abandonded => &mut self.abandoned,
            TriggerName::Banished => &mut self.banished,
            TriggerName::Discarded => &mut self.discarded,
            TriggerName::Dissolved => &mut self.dissolved,
            TriggerName::DrewAllCardsInCopyOfDeck => &mut self.drew_all_cards_in_copy_of_deck,
            TriggerName::EndOfTurn => &mut self.end_of_turn,
            TriggerName::GainedEnergy => &mut self.gained_energy,
            TriggerName::Judgment => &mut self.judgment,
            TriggerName::Materialized => &mut self.materialized,
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
