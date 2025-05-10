use std::collections::{BTreeSet, VecDeque};

use core_data::identifiers::CardIdent;
use core_data::types::PlayerName;
use rand::seq::SliceRandom;
use rand_xoshiro::Xoshiro256PlusPlus;
use slotmap::SlotMap;

use crate::battle_cards::card_data::CardData;
use crate::battle_cards::card_id::{
    BanishedCardId, CardIdType, CharacterId, DeckCardId, HandCardId, ObjectId, StackCardId,
    VoidCardId,
};
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    cards: SlotMap<CardIdent, CardData>,
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
        self.cards.get(id.card_id())
    }

    /// Mutable equivalent of [Self::card]
    pub fn card_mut(&mut self, id: impl CardIdType) -> Option<&mut CardData> {
        self.cards.get_mut(id.card_id())
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
        self.battlefield.cards(player_name).iter().map(|id| &self.cards[id.card_id()])
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

    /// Returns the card at the top of the stack, if there is one.
    pub fn top_of_stack(&self) -> Option<&CardData> {
        self.card(*self.stack.last()?)
    }

    /// Mutable equivalent of [Self::top_of_stack].
    pub fn top_of_stack_mut(&mut self) -> Option<&mut CardData> {
        self.card_mut(*self.stack.last()?)
    }

    /// Returns the set of cards in the hand for a given player.
    pub fn hand(&self, player_name: PlayerName) -> &BTreeSet<HandCardId> {
        self.hand.cards(player_name)
    }

    /// Returns an iterator over all characters on the battlefield for a given
    /// player.
    pub fn hand_cards(&self, player_name: PlayerName) -> impl Iterator<Item = &CardData> {
        self.hand.cards(player_name).iter().map(|id| &self.cards[id.card_id()])
    }

    /// Returns the set of banished cards for a given player.
    pub fn banished(&self, player_name: PlayerName) -> &BTreeSet<BanishedCardId> {
        self.banished.cards(player_name)
    }

    /// Creates a card instance in its associated zone and assigns a [CardIdent]
    /// to  it. The expected way to call this method is by passing a [CardData]
    /// which has been assigned the default CardId.
    ///
    /// This does *not* make the card revealed to any player.
    pub fn create_card(&mut self, card_data: CardData) -> CardIdent {
        let zone = card_data.zone;
        let owner = card_data.owner;
        let card_id = self.cards.insert(card_data);
        self.cards[card_id].id = card_id;
        self.add_to_zone(owner, card_id, zone);
        card_id
    }

    /// Moves a card from its current zone to a new zone, if it is present.
    /// Generally you should use the `move_card` module instead of invoking this
    /// directly.
    ///
    /// Returns the new ObjectId for the card if a moved occurred, or None if
    /// the card was not found.
    pub fn move_card(&mut self, card_id: impl CardIdType, to: Zone) -> Option<ObjectId> {
        let id = card_id.card_id();
        let card = self.card(id)?;
        let owner = card.owner;
        self.remove_from_zone(owner, card.zone, card.id);
        self.add_to_zone(owner, id, to)
    }

    /// Shuffles the deck for a given player.
    pub fn shuffle_deck(&mut self, player: PlayerName, rng: &mut Xoshiro256PlusPlus) {
        self.deck.shuffle(player, rng);
    }

    /// Randomizes the hand of the provided `player` and both players' decks.
    ///
    /// - Moves all cards from this player's hand to their deck.
    /// - Shuffles both decks
    /// - Returns an equivalent number random of cards from this player's deck
    ///   to their hand.
    pub fn randomize_player(&mut self, player: PlayerName, rng: &mut Xoshiro256PlusPlus) {
        let hand_cards: Vec<HandCardId> = self.hand(player).iter().copied().collect();
        let hand_count = hand_cards.len();

        for card_id in hand_cards {
            self.move_card(card_id, Zone::Deck);
        }

        self.shuffle_deck(PlayerName::One, rng);
        self.shuffle_deck(PlayerName::Two, rng);

        for _ in 0..hand_count {
            if let Some(&deck_card) = self.deck(player).front() {
                self.move_card(deck_card, Zone::Hand);
            }
        }
    }

    fn new_object_id(&mut self) -> ObjectId {
        let result = self.next_object_id;
        self.next_object_id = ObjectId(result.0 + 1);
        result
    }

    fn add_to_zone(
        &mut self,
        owner: PlayerName,
        card_id: CardIdent,
        zone: Zone,
    ) -> Option<ObjectId> {
        let object_id = self.new_object_id();
        self.card_mut(card_id)?.zone = zone;
        self.card_mut(card_id)?.object_id = object_id;

        match zone {
            Zone::Banished => self.banished.add(owner, BanishedCardId(card_id)),
            Zone::Battlefield => self.battlefield.add(owner, CharacterId(card_id)),
            Zone::Deck => self.deck.add(owner, DeckCardId(card_id)),
            Zone::Hand => self.hand.add(owner, HandCardId(card_id)),
            Zone::Stack => self.stack.push(StackCardId(card_id)),
            Zone::Void => self.void.add(owner, VoidCardId(card_id)),
        }

        Some(object_id)
    }

    fn remove_from_zone(&mut self, owner: PlayerName, zone: Zone, card_id: CardIdent) {
        match zone {
            Zone::Banished => {
                self.banished.remove_if_present(owner, BanishedCardId(card_id));
            }
            Zone::Battlefield => {
                self.battlefield.remove_if_present(owner, CharacterId(card_id));
            }
            Zone::Deck => {
                self.deck.remove_if_present(owner, DeckCardId(card_id));
            }
            Zone::Hand => {
                self.hand.remove_if_present(owner, HandCardId(card_id));
            }
            Zone::Stack => {
                if let Some((i, _)) = self
                    .stack
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, &stack_id)| card_id == stack_id.card_id())
                {
                    self.stack.remove(i);
                }
            }
            Zone::Void => {
                self.void.remove_if_present(owner, VoidCardId(card_id));
            }
        }
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
            PlayerName::One => &self.user,
            PlayerName::Two => &self.enemy,
        }
    }

    pub fn cards_mut(&mut self, player_name: PlayerName) -> &mut BTreeSet<T> {
        match player_name {
            PlayerName::One => &mut self.user,
            PlayerName::Two => &mut self.enemy,
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
            PlayerName::One => &self.user,
            PlayerName::Two => &self.enemy,
        }
    }

    pub fn cards_mut(&mut self, player_name: PlayerName) -> &mut VecDeque<T> {
        match player_name {
            PlayerName::One => &mut self.user,
            PlayerName::Two => &mut self.enemy,
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

    pub fn shuffle(&mut self, player: PlayerName, rng: &mut Xoshiro256PlusPlus) {
        let cards = self.cards_mut(player);
        cards.make_contiguous().shuffle(rng);
    }
}

impl<T: CardIdType> Default for OrderedZone<T> {
    fn default() -> Self {
        Self { user: VecDeque::new(), enemy: VecDeque::new() }
    }
}
