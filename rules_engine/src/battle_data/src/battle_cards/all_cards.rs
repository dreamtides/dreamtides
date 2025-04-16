use std::collections::{BTreeSet, VecDeque};

use ability_data::ability::Ability;
use core_data::identifiers::CardId;
use core_data::types::PlayerName;
use slotmap::SlotMap;

use crate::battle_cards::card_data::CardData;
use crate::battle_cards::card_id::{
    BanishedCardId, CardIdType, CharacterId, DeckCardId, HandCardId, ObjectId, StackCardId,
    VoidCardId,
};
use crate::battle_cards::card_instance_id::CardInstanceId;
use crate::battle_cards::card_properties::CardProperties;
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    cards: SlotMap<CardId, CardData>,
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
    pub fn card(&self, id: impl CardIdType) -> Option<&CardData> {
        self.cards.get(id.card_identifier(self)?)
    }

    /// Mutable equivalent of [Self::card]
    pub fn card_mut(&mut self, id: impl CardIdType) -> Option<&mut CardData> {
        self.cards.get_mut(id.card_identifier(self)?)
    }

    /// Returns all currently known cards in an undefined order
    pub fn all_cards(&self) -> impl Iterator<Item = &CardData> {
        self.cards.values()
    }

    /// Returns the set of characters on the battlefield for a given player.
    pub fn battlefield(&self, player_name: PlayerName) -> &BTreeSet<CharacterId> {
        self.battlefield.cards(player_name)
    }

    /// Returns an iterator over all characters on the battlefield for a given
    /// player.
    pub fn battlefield_cards(&self, player_name: PlayerName) -> impl Iterator<Item = &CardData> {
        self.battlefield.cards(player_name).iter().map(|id| &self.cards[id.0.card_id])
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

    /// Returns an iterator over all characters on the battlefield for a given
    /// player.
    pub fn hand_cards(&self, player_name: PlayerName) -> impl Iterator<Item = &CardData> {
        self.hand.cards(player_name).iter().map(|id| &self.cards[id.0.card_id])
    }

    /// Returns the set of banished cards for a given player.
    pub fn banished(&self, player_name: PlayerName) -> &BTreeSet<BanishedCardId> {
        self.banished.cards(player_name)
    }

    /// Creates a card instance in the given zone.
    ///
    /// This does *not* make the card revealed to any player.
    pub fn create_card(
        &mut self,
        owner: PlayerName,
        zone: Zone,
        properties: CardProperties,
        abilities: Vec<Ability>,
    ) -> CardId {
        let object_id = self.new_object_id();
        let tmp_instance_id = create_card_instance_id(zone, object_id, CardId::default());
        let card_data_id =
            self.cards.insert(CardData::new(tmp_instance_id, owner, properties, abilities));
        self.cards[card_data_id].internal_set_id(create_card_instance_id(
            zone,
            object_id,
            card_data_id,
        ));
        self.add_to_zone(owner, card_data_id, object_id, zone);
        card_data_id
    }

    /// Moves a card from its current zone to a new zone, if it is present.
    /// Generally you should use the `move_card` module instead of invoking this
    /// directly.
    ///
    /// Returns the new ObjectId for the card if a moved occurred, or None if
    /// the card was not found.
    pub fn move_card(&mut self, card_id: impl CardIdType, to: Zone) -> Option<ObjectId> {
        let card_data_id = card_id.card_identifier(self)?;
        let card = self.card(card_data_id)?;
        let owner = card.owner;
        self.remove_from_zone(owner, card.id);
        let object_id = self.new_object_id();
        self.add_to_zone(owner, card_data_id, object_id, to)?;
        Some(object_id)
    }

    fn new_object_id(&mut self) -> ObjectId {
        let result = self.next_object_id;
        self.next_object_id = ObjectId(result.0 + 1);
        result
    }

    fn add_to_zone(
        &mut self,
        owner: PlayerName,
        card_id: CardId,
        new_object_id: ObjectId,
        zone: Zone,
    ) -> Option<()> {
        let instance_id = create_card_instance_id(zone, new_object_id, card_id);
        self.card_mut(card_id)?.internal_set_id(instance_id);

        match instance_id {
            CardInstanceId::Banished(id) => self.banished.add(owner, id),
            CardInstanceId::Battlefield(id) => self.battlefield.add(owner, id),
            CardInstanceId::Deck(id) => self.deck.add(owner, id),
            CardInstanceId::Hand(id) => self.hand.add(owner, id),
            CardInstanceId::Stack(id) => self.stack.push(id),
            CardInstanceId::Void(id) => self.void.add(owner, id),
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

fn create_card_instance_id(zone: Zone, object_id: ObjectId, card_id: CardId) -> CardInstanceId {
    match zone {
        Zone::Banished => CardInstanceId::Banished(BanishedCardId::new(object_id, card_id)),
        Zone::Battlefield => CardInstanceId::Battlefield(CharacterId::new(object_id, card_id)),
        Zone::Deck => CardInstanceId::Deck(DeckCardId::new(object_id, card_id)),
        Zone::Hand => CardInstanceId::Hand(HandCardId::new(object_id, card_id)),
        Zone::Stack => CardInstanceId::Stack(StackCardId::new(object_id, card_id)),
        Zone::Void => CardInstanceId::Void(VoidCardId::new(object_id, card_id)),
    }
}

#[derive(Debug, Clone)]
struct UnorderedZone<T> {
    user: BTreeSet<T>,
    enemy: BTreeSet<T>,
}

impl<T: CardIdType> UnorderedZone<T> {
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

impl<T: CardIdType> Default for UnorderedZone<T> {
    fn default() -> Self {
        Self { user: BTreeSet::new(), enemy: BTreeSet::new() }
    }
}

#[derive(Debug, Clone)]
struct OrderedZone<T> {
    user: VecDeque<T>,
    enemy: VecDeque<T>,
}

impl<T: CardIdType> OrderedZone<T> {
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

impl<T: CardIdType> Default for OrderedZone<T> {
    fn default() -> Self {
        Self { user: VecDeque::new(), enemy: VecDeque::new() }
    }
}
