use std::collections::{BTreeSet, VecDeque};

use core_data::identifiers::CardDataIdentifier;
use core_data::types::PlayerName;
use slotmap::SlotMap;

use crate::cards::card_data::CardData;
use crate::cards::card_id::{
    BanishedCardId, CardId, CharacterId, DeckCardId, HandCardId, ObjectId, StackCardId, VoidCardId,
};
use crate::cards::card_instance_id::CardInstanceId;
use crate::cards::zone::Zone;

#[derive(Clone, Debug)]
pub struct AllCards {
    cards: SlotMap<CardDataIdentifier, CardData>,
    battlefield: UnorderedZone<CharacterId>,
    void: UnorderedZone<VoidCardId>,
    hand: UnorderedZone<HandCardId>,
    deck: OrderedZone<DeckCardId>,
    stack: Vec<StackCardId>,
    banished: UnorderedZone<BanishedCardId>,
    next_object_id: ObjectId,
}

impl AllCards {
    /// Looks up the state for a card.
    ///
    /// Returns None if this card or id no longer exists, e.g. if it's the ID of
    /// a token which has been destroyed, a permanent which is no longer on the
    /// battlefield, etc.
    pub fn card(&self, id: impl CardId) -> Option<&CardData> {
        self.cards.get(id.card_identifier(self)?)
    }

    /// Mutable equivalent of [Self::card]
    pub fn card_mut(&mut self, id: impl CardId) -> Option<&mut CardData> {
        self.cards.get_mut(id.card_identifier(self)?)
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

    /// Allocates a new ObjectId to track instances within zones.
    pub fn new_object_id(&mut self) -> ObjectId {
        let result = self.next_object_id;
        self.next_object_id = ObjectId(result.0 + 1);
        result
    }

    /// Moves a card from its current zone to a new zone, if it is present.
    ///
    /// Returns the new ObjectId for the card if a moved occurred, or None if
    /// the card was not found.
    pub fn move_card(&mut self, card_id: impl CardId, to: Zone) -> Option<ObjectId> {
        let card_data_id = card_id.card_identifier(self)?;
        let card = self.card(card_data_id)?;
        let owner = card.owner;
        self.remove_from_zone(owner, card.id);
        let object_id = self.new_object_id();
        self.add_to_zone(owner, card_data_id, object_id, to)?;
        Some(object_id)
    }

    fn add_to_zone(
        &mut self,
        owner: PlayerName,
        card_id: CardDataIdentifier,
        new_object_id: ObjectId,
        zone: Zone,
    ) -> Option<()> {
        match zone {
            Zone::Banished => {
                let id = BanishedCardId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Banished(id);
                self.banished.add(owner, id);
            }
            Zone::Battlefield => {
                let id = CharacterId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Battlefield(id);
                self.battlefield.add(owner, id);
            }
            Zone::Deck => {
                let id = DeckCardId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Deck(id);
                self.deck.add(owner, id);
            }
            Zone::Hand => {
                let id = HandCardId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Hand(id);
                self.hand.add(owner, id);
            }
            Zone::Stack => {
                let id = StackCardId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Stack(id);
                self.stack.push(id);
            }
            Zone::Void => {
                let id = VoidCardId::new(new_object_id, card_id);
                self.card_mut(id)?.id = CardInstanceId::Void(id);
                self.void.add(owner, id);
            }
        }

        Some(())
    }

    fn remove_from_zone(&mut self, owner: PlayerName, instance_id: CardInstanceId) {
        match instance_id {
            CardInstanceId::Banished(id) => {
                self.banished.remove_if_present(owner, id);
            }
            CardInstanceId::Battlefield(id) => {
                self.battlefield.remove_if_present(owner, id);
            }
            CardInstanceId::Deck(id) => {
                self.deck.remove_if_present(owner, id);
            }
            CardInstanceId::Hand(id) => {
                self.hand.remove_if_present(owner, id);
            }
            CardInstanceId::Stack(id) => {
                if let Some((i, _)) =
                    self.stack.iter().enumerate().rev().find(|(_, &stack_id)| id == stack_id)
                {
                    self.stack.remove(i);
                }
            }
            CardInstanceId::Void(id) => {
                self.void.remove_if_present(owner, id);
            }
        }
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

    pub fn cards_mut(&mut self, player_name: PlayerName) -> &mut BTreeSet<T> {
        match player_name {
            PlayerName::User => &mut self.user,
            PlayerName::Enemy => &mut self.enemy,
        }
    }

    pub fn add(&mut self, owner: PlayerName, card_id: T) {
        self.cards_mut(owner).insert(card_id);
    }

    pub fn remove_if_present(&mut self, owner: PlayerName, card_id: T) -> Option<T> {
        self.cards_mut(owner).remove(&card_id).then_some(card_id)
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

    pub fn cards_mut(&mut self, player_name: PlayerName) -> &mut VecDeque<T> {
        match player_name {
            PlayerName::User => &mut self.user,
            PlayerName::Enemy => &mut self.enemy,
        }
    }

    pub fn add(&mut self, owner: PlayerName, card_id: T) {
        self.cards_mut(owner).push_back(card_id);
    }

    pub fn remove_if_present(&mut self, owner: PlayerName, card_id: T) -> Option<T> {
        if let Some((i, _)) =
            self.cards_mut(owner).iter().enumerate().rev().find(|(_, &id)| id == card_id)
        {
            self.cards_mut(owner).remove(i)
        } else {
            None
        }
    }
}
