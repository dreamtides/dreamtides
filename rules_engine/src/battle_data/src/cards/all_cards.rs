use std::collections::{BTreeSet, VecDeque};

use core_data::identifiers::CardDataIdentifier;
use core_data::types::PlayerName;
use slotmap::SlotMap;

use crate::cards::card_data::CardData;
use crate::cards::card_id::{
    BanishedCardId, CardId, CharacterId, DeckCardId, HandCardId, StackCardId, VoidCardId,
};

#[derive(Clone, Debug)]
pub struct AllCards {
    cards: SlotMap<CardDataIdentifier, CardData>,
    battlefield: UnorderedZone<CharacterId>,
    void: UnorderedZone<VoidCardId>,
    hand: UnorderedZone<HandCardId>,
    deck: OrderedZone<DeckCardId>,
    stack: Vec<StackCardId>,
    banished: UnorderedZone<BanishedCardId>,
}

impl AllCards {
    /// Looks up the state for a card.
    ///
    /// Returns None if this card or id no longer exists, e.g. if it's the ID of
    /// a token which has been destroyed, a permanent which is no longer on the
    /// battlefield, etc.
    pub fn card(&self, id: impl CardId) -> Option<&CardData> {
        self.cards.get(id.internal_card_identifier(self)?)
    }

    /// Mutable equivalent of [Self::card]
    pub fn card_mut(&mut self, id: impl CardId) -> Option<&mut CardData> {
        self.cards.get_mut(id.internal_card_identifier(self)?)
    }

    /// Returns the set of characters on the battlefield for a given player.
    pub fn battlefield(&self, player_name: PlayerName) -> &BTreeSet<CharacterId> {
        self.battlefield.cards(player_name)
    }

    /// Returns the set of cards in the void for a given player.
    pub fn void(&self, player_name: PlayerName) -> &BTreeSet<VoidCardId> {
        self.void.cards(player_name)
    }

    /// Returns the set of cards in the deck for a given player.
    pub fn deck(&self, player_name: PlayerName) -> &VecDeque<DeckCardId> {
        self.deck.cards(player_name)
    }

    /// Returns the IDs of cards on the stack
    pub fn stack(&self) -> &[StackCardId] {
        &self.stack
    }

    /// Returns the set of cards in the hand for a given player.
    pub fn hand(&self, player_name: PlayerName) -> &BTreeSet<HandCardId> {
        self.hand.cards(player_name)
    }

    /// Returns the set of banished cards for a given player.
    pub fn banished(&self, player_name: PlayerName) -> &BTreeSet<BanishedCardId> {
        self.banished.cards(player_name)
    }
}

#[derive(Default, Debug, Clone)]
struct UnorderedZone<T> {
    user: BTreeSet<T>,
    enemy: BTreeSet<T>,
}

impl<T: CardId> UnorderedZone<T> {
    pub fn cards(&self, player_name: PlayerName) -> &BTreeSet<T> {
        match player_name {
            PlayerName::User => &self.user,
            PlayerName::Enemy => &self.enemy,
        }
    }
}

#[derive(Default, Debug, Clone)]
struct OrderedZone<T> {
    user: VecDeque<T>,
    enemy: VecDeque<T>,
}

impl<T: CardId> OrderedZone<T> {
    pub fn cards(&self, player_name: PlayerName) -> &VecDeque<T> {
        match player_name {
            PlayerName::User => &self.user,
            PlayerName::Enemy => &self.enemy,
        }
    }
}
