use std::collections::BTreeMap;

use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::battle::card_id::{
    BanishedCardId, CardId, CardIdType, CharacterId, DeckCardId, HandCardId, StackCardId,
    VoidCardId,
};
use crate::battle_cards::ability_list::CanPlayRestriction;
use crate::battle_cards::battle_card_state::{BattleCardState, ObjectId};
use crate::battle_cards::card_set::CardSet;
use crate::battle_cards::character_state::CharacterState;
use crate::battle_cards::stack_card_state::{StackCardAdditionalCostsPaid, StackCardState};
use crate::battle_cards::zone::Zone;
use crate::battle_player::player_map::PlayerMap;

/// A map of characters on the battlefield to their states
///
/// No significant differences between BTreeMap and SmallMap here.
pub type CharacterMap = BTreeMap<CharacterId, CharacterState>;

/// A vector of cards on the stack
///
/// No significant differences between SmallVec and Vec here.
pub type StackCards = Vec<StackCardState>;

/// A card to create in a player's deck.
pub struct CreatedCard {
    pub name: CardName,
    pub can_play_restriction: Option<CanPlayRestriction>,
}

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    cards: Vec<BattleCardState>,
    battlefield: PlayerMap<CardSet<CharacterId>>,
    battlefield_state: PlayerMap<CharacterMap>,
    void: PlayerMap<CardSet<VoidCardId>>,
    hands: PlayerMap<CardSet<HandCardId>>,
    decks: PlayerMap<CardSet<DeckCardId>>,
    stack: StackCards,
    stack_set: PlayerMap<CardSet<StackCardId>>,
    banished: PlayerMap<CardSet<BanishedCardId>>,
    next_object_id: ObjectId,
}

impl AllCards {
    /// Returns the state of a card without bounds checking.
    ///
    /// # Safety
    /// Always use `card::get` in battle_queries instead of this function,
    /// because it performs bounds checking via [Self::is_valid_card_id].
    #[inline(always)]
    pub unsafe fn get_card_unchecked(&self, id: CardId) -> &BattleCardState {
        unsafe { self.cards.get_unchecked(id.0) }
    }

    /// Returns the spark value of a character.
    ///
    /// Returns None if this character is not present on the battlefield.
    pub fn spark(&self, controller: PlayerName, id: CharacterId) -> Option<Spark> {
        self.battlefield_state
            .player(controller)
            .get(&id)
            .map(|character_state| character_state.spark)
    }

    /// Returns the set of cards in a player's hand.
    pub fn hand(&self, player: PlayerName) -> &CardSet<HandCardId> {
        self.hands.player(player)
    }

    /// Returns the set of cards in a player's deck.
    pub fn deck(&self, player: PlayerName) -> &CardSet<DeckCardId> {
        self.decks.player(player)
    }

    /// Returns the set of characters on the battlefield for a given player
    pub fn battlefield(&self, player: PlayerName) -> &CardSet<CharacterId> {
        self.battlefield.player(player)
    }

    /// Returns the state of characters on the battlefield for a given player
    pub fn battlefield_state(&self, player: PlayerName) -> &CharacterMap {
        self.battlefield_state.player(player)
    }

    /// Mutable equivalent to [Self::battlefield_state]
    pub fn battlefield_state_mut(&mut self, player: PlayerName) -> &mut CharacterMap {
        self.battlefield_state.player_mut(player)
    }

    /// Returns the set of cards in a player's void
    pub fn void(&self, player: PlayerName) -> &CardSet<VoidCardId> {
        self.void.player(player)
    }

    /// Returns true if a stack is currently active.
    pub fn has_stack(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Returns the state of a card on the stack, if any.
    pub fn stack_card(&self, id: StackCardId) -> Option<&StackCardState> {
        self.stack.iter().rev().find(|card| card.id == id)
    }

    /// Returns the top card on the stack, if any.
    pub fn top_of_stack(&self) -> Option<&StackCardState> {
        self.stack.last()
    }

    /// Mutable equivalent to [Self::top_of_stack].
    pub fn top_of_stack_mut(&mut self) -> Option<&mut StackCardState> {
        self.stack.last_mut()
    }

    /// Returns all cards currently on the stack.
    pub fn all_cards_on_stack(&self) -> &StackCards {
        &self.stack
    }

    /// Returns the set of cards on the stack for a given player.
    pub fn stack_set(&self, player: PlayerName) -> &CardSet<StackCardId> {
        self.stack_set.player(player)
    }

    /// Returns the set of banished cards for a given player.
    pub fn banished(&self, player: PlayerName) -> &CardSet<BanishedCardId> {
        self.banished.player(player)
    }

    /// Returns all currently known Card IDs in an undefined order
    pub fn all_cards(&self) -> impl Iterator<Item = CardId> + '_ {
        self.cards.iter().enumerate().map(|(i, _)| CardId(i))
    }

    /// Creates a set of cards with the indicated names in a player's deck.
    pub fn create_cards_in_deck(&mut self, owner: PlayerName, cards: Vec<CreatedCard>) {
        for name in cards {
            let id = self.cards.len();
            let object_id = self.new_object_id();
            self.cards.push(BattleCardState {
                name: name.name,
                owner,
                object_id,
                can_play_restriction: name.can_play_restriction,
            });
            self.decks.player_mut(owner).insert(DeckCardId(CardId(id)));
        }
    }

    /// Moves a card from its current zone to a new zone, if it is present.
    /// Generally you should use the `move_card` module instead of invoking this
    /// directly.
    ///
    /// This *only* updates the position of the card, and writes the default
    /// card state values in its new zone (e.g. 0 spark for a character on the
    /// battlefield). You should write a correct state value for the new zone if
    /// appropriate.
    pub fn move_card(
        &mut self,
        controller: PlayerName,
        card_id: impl CardIdType,
        from: Zone,
        to: Zone,
    ) {
        let id = card_id.card_id();
        self.remove_from_zone(controller, id, from);
        self.add_to_zone(controller, id, to);
        self.cards[id.0].object_id = self.new_object_id();
    }

    /// Returns true if the indicated card is present in the indicated zone.
    pub fn contains_card(&self, controller: PlayerName, card_id: CardId, zone: Zone) -> bool {
        match zone {
            Zone::Banished => {
                self.banished.player(controller).contains(BanishedCardId(CardId(card_id.0)))
            }
            Zone::Battlefield => self.battlefield.player(controller).contains(CharacterId(card_id)),
            Zone::Deck => self.decks.player(controller).contains(DeckCardId(CardId(card_id.0))),
            Zone::Hand => self.hands.player(controller).contains(HandCardId(CardId(card_id.0))),
            Zone::Stack => {
                self.stack_set.player(controller).contains(StackCardId(CardId(card_id.0)))
            }
            Zone::Void => self.void.player(controller).contains(VoidCardId(CardId(card_id.0))),
        }
    }

    /// Returns true if the indicated card has the indicated object ID.
    ///
    /// Panics if no card with this ID exists.
    pub fn is_valid_object_id(&self, card_id: impl CardIdType, object_id: ObjectId) -> bool {
        self.cards[card_id.card_id().0].object_id == object_id
    }

    /// Returns the character ID for a card, if it is currently a character.
    pub fn to_character_id(
        &self,
        controller: PlayerName,
        card_id: impl CardIdType,
    ) -> Option<CharacterId> {
        let result = CharacterId(card_id.card_id());
        if self.battlefield(controller).contains(result) { Some(result) } else { None }
    }

    /// Returns true if the indicated card ID is valid.
    #[inline(always)]
    pub fn is_valid_card_id(&self, card_id: CardId) -> bool {
        card_id.0 < self.cards.len()
    }

    fn new_object_id(&mut self) -> ObjectId {
        let result = self.next_object_id;
        self.next_object_id = ObjectId(result.0 + 1);
        result
    }

    fn add_to_zone(&mut self, controller: PlayerName, card_id: CardId, zone: Zone) {
        match zone {
            Zone::Banished => {
                self.banished.player_mut(controller).insert(BanishedCardId(card_id));
            }
            Zone::Battlefield => {
                self.battlefield.player_mut(controller).insert(CharacterId(card_id));
                self.battlefield_state
                    .player_mut(controller)
                    .insert(CharacterId(card_id), CharacterState::default());
            }
            Zone::Deck => {
                self.decks.player_mut(controller).insert(DeckCardId(card_id));
            }
            Zone::Hand => {
                self.hands.player_mut(controller).insert(HandCardId(card_id));
            }
            Zone::Stack => {
                self.stack_set.player_mut(controller).insert(StackCardId(card_id));
                self.stack.push(StackCardState {
                    id: StackCardId(card_id),
                    controller,
                    targets: None,
                    additional_costs_paid: StackCardAdditionalCostsPaid::None,
                });
            }
            Zone::Void => {
                self.void.player_mut(controller).insert(VoidCardId(card_id));
            }
        }
    }

    fn remove_from_zone(&mut self, controller: PlayerName, card_id: CardId, zone: Zone) {
        match zone {
            Zone::Banished => {
                self.banished.player_mut(controller).remove(BanishedCardId(card_id));
            }
            Zone::Battlefield => {
                self.battlefield.player_mut(controller).remove(CharacterId(card_id));
                self.battlefield_state.player_mut(controller).remove(&CharacterId(card_id));
            }
            Zone::Deck => {
                self.decks.player_mut(controller).remove(DeckCardId(card_id));
            }
            Zone::Hand => {
                self.hands.player_mut(controller).remove(HandCardId(card_id));
            }
            Zone::Stack => {
                self.stack_set.player_mut(controller).remove(StackCardId(card_id));
                if let Some((i, _)) = self
                    .stack
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, stack_card)| StackCardId(card_id) == stack_card.id)
                {
                    self.stack.remove(i);
                }
            }
            Zone::Void => {
                self.void.player_mut(controller).remove(VoidCardId(card_id));
            }
        }
    }
}
