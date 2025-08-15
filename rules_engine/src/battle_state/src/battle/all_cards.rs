use std::collections::BTreeMap;

use core_data::card_types::CardType;
use core_data::identifiers::CardIdentity;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;

use crate::battle::card_id::{
    ActivatedAbilityId, BanishedCardId, CardId, CardIdType, CharacterId, DeckCardId, HandCardId,
    StackCardId, VoidCardId,
};
use crate::battle_cards::ability_list::CanPlayRestriction;
use crate::battle_cards::battle_card_state::{BattleCardState, ObjectId};
use crate::battle_cards::card_set::CardSet;
use crate::battle_cards::character_state::CharacterState;
use crate::battle_cards::stack_card_state::{
    StackCardAdditionalCostsPaid, StackItemId, StackItemState, StackItems,
};
use crate::battle_cards::zone::Zone;
use crate::battle_player::player_map::PlayerMap;

/// A map of characters on the battlefield to their states
///
/// No significant differences between BTreeMap and SmallMap here.
pub type CharacterMap = BTreeMap<CharacterId, CharacterState>;

/// A card to create in a player's deck.
pub struct CreatedCard {
    pub identity: CardIdentity,
    pub can_play_restriction: Option<CanPlayRestriction>,
    pub base_energy_cost: Option<Energy>,
    pub base_spark: Option<Spark>,
    pub card_type: CardType,
    pub is_fast: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    cards: Vec<BattleCardState>,
    battlefield: PlayerMap<CardSet<CharacterId>>,
    battlefield_state: PlayerMap<CharacterMap>,
    void: PlayerMap<CardSet<VoidCardId>>,
    hands: PlayerMap<CardSet<HandCardId>>,
    shuffled_into_decks: PlayerMap<CardSet<DeckCardId>>,
    tops_of_decks: PlayerMap<Vec<DeckCardId>>,
    stack: StackItems,
    stack_card_set: PlayerMap<CardSet<StackCardId>>,
    banished: PlayerMap<CardSet<BanishedCardId>>,
    next_object_id: ObjectId,
    activated_ability_object_ids: BTreeMap<ActivatedAbilityId, ObjectId>,
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

    /// Mutable equivalent to [Self::get_card_unchecked]
    ///
    /// # Safety
    /// Always use `card::get_mut` in battle_mutations instead of this function,
    /// because it performs bounds checking via [Self::is_valid_card_id].
    #[inline(always)]
    pub unsafe fn get_card_unchecked_mut(&mut self, id: CardId) -> &mut BattleCardState {
        unsafe { self.cards.get_unchecked_mut(id.0) }
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

    /// Returns the set of cards shuffled into a player's deck.
    pub fn shuffled_into_deck(&self, player: PlayerName) -> &CardSet<DeckCardId> {
        self.shuffled_into_decks.player(player)
    }

    /// Returns the top of deck cards for a given player.
    ///
    /// The last element of the vector is the topmost card of the deck.
    pub fn top_of_deck(&self, player: PlayerName) -> &Vec<DeckCardId> {
        self.tops_of_decks.player(player)
    }

    /// Mutable equivalent to [Self::top_of_deck].
    ///
    /// The last element of the vector is the topmost card of the deck.
    pub fn top_of_deck_mut(&mut self, player: PlayerName) -> &mut Vec<DeckCardId> {
        self.tops_of_decks.player_mut(player)
    }

    /// Returns all cards in a player's deck.
    pub fn all_deck_cards(&self, player: PlayerName) -> impl Iterator<Item = DeckCardId> + '_ {
        self.shuffled_into_decks
            .player(player)
            .iter()
            .chain(self.tops_of_decks.player(player).iter().copied())
    }

    /// Returns the set of characters on the battlefield for a given player
    pub fn battlefield(&self, player: PlayerName) -> &CardSet<CharacterId> {
        self.battlefield.player(player)
    }

    /// Returns an iterator over all characters on the battlefield.
    pub fn all_battlefield_characters(&self) -> impl Iterator<Item = CharacterId> + '_ {
        self.battlefield.one.iter().chain(self.battlefield.two.iter())
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

    /// Returns the state of an item on the stack, if any.
    pub fn stack_item(&self, id: impl Into<StackItemId>) -> Option<&StackItemState> {
        let id = id.into();
        self.stack.iter().rev().find(|item| item.id == id)
    }

    /// Mutable equivalent to [Self::stack_item]
    pub fn stack_item_mut(&mut self, id: impl Into<StackItemId>) -> Option<&mut StackItemState> {
        let id = id.into();
        self.stack.iter_mut().rev().find(|item| item.id == id)
    }

    /// Returns the top item on the stack, if any.
    pub fn top_of_stack(&self) -> Option<&StackItemState> {
        self.stack.last()
    }

    /// Mutable equivalent to [Self::top_of_stack].
    pub fn top_of_stack_mut(&mut self) -> Option<&mut StackItemState> {
        self.stack.last_mut()
    }

    /// Returns all cards currently on the stack.
    pub fn all_items_on_stack(&self) -> &StackItems {
        &self.stack
    }

    /// Returns the set of cards on the stack for a given player.
    pub fn stack_set(&self, player: PlayerName) -> &CardSet<StackCardId> {
        self.stack_card_set.player(player)
    }

    /// Returns the set of banished cards for a given player.
    pub fn banished(&self, player: PlayerName) -> &CardSet<BanishedCardId> {
        self.banished.player(player)
    }

    /// Moves a card from deck to the topmost position in the deck for a given
    /// player.
    ///
    /// Returns true if the card was found and moved.
    pub fn move_card_to_top_of_deck(&mut self, player: PlayerName, card_id: DeckCardId) -> bool {
        if self.shuffled_into_decks.player(player).contains(card_id) {
            self.shuffled_into_decks.player_mut(player).remove(card_id);
            self.tops_of_decks.player_mut(player).push(card_id);
            true
        } else {
            false
        }
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
                identity: name.identity,
                owner,
                object_id,
                base_energy_cost: name.base_energy_cost,
                base_spark: name.base_spark,
                card_type: name.card_type,
                is_fast: name.is_fast,
                revealed_to_player_override: PlayerMap::default(),
                can_play_restriction: name.can_play_restriction,
            });
            self.shuffled_into_decks.player_mut(owner).insert(DeckCardId(CardId(id)));
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

        let new_object_id = self.new_object_id();
        let state = &mut self.cards[id.0];
        state.object_id = new_object_id;
        state.revealed_to_player_override = PlayerMap::default();
    }

    /// Returns true if the indicated card is present in the indicated zone.
    pub fn contains_card(&self, controller: PlayerName, card_id: CardId, zone: Zone) -> bool {
        match zone {
            Zone::Banished => {
                self.banished.player(controller).contains(BanishedCardId(CardId(card_id.0)))
            }
            Zone::Battlefield => self.battlefield.player(controller).contains(CharacterId(card_id)),
            Zone::Deck => {
                self.shuffled_into_decks.player(controller).contains(DeckCardId(CardId(card_id.0)))
                    || self
                        .tops_of_decks
                        .player(controller)
                        .contains(&DeckCardId(CardId(card_id.0)))
            }
            Zone::Hand => self.hands.player(controller).contains(HandCardId(CardId(card_id.0))),
            Zone::Stack => {
                self.stack_card_set.player(controller).contains(StackCardId(CardId(card_id.0)))
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

    /// Returns the stack card ID for a card, if it is currently on the stack.
    pub fn to_stack_card_id(
        &self,
        controller: PlayerName,
        card_id: impl CardIdType,
    ) -> Option<StackCardId> {
        let result = StackCardId(card_id.card_id());
        if self.stack_set(controller).contains(result) { Some(result) } else { None }
    }

    /// Returns the void card ID for a card, if it is currently in the void.
    pub fn to_void_card_id(
        &self,
        controller: PlayerName,
        card_id: impl CardIdType,
    ) -> Option<VoidCardId> {
        let result = VoidCardId(card_id.card_id());
        if self.void(controller).contains(result) { Some(result) } else { None }
    }

    /// Returns true if the indicated card ID is valid.
    #[inline(always)]
    pub fn is_valid_card_id(&self, card_id: CardId) -> bool {
        card_id.0 < self.cards.len()
    }

    /// Returns the next object ID to use for a card in the display layer. This
    /// is intended to render purely visual cards like triggered abilities on
    /// top of all 'real' cards.
    pub fn next_object_id_for_display(&self) -> ObjectId {
        self.next_object_id
    }

    /// Returns the object ID for an activated ability, if it is currently on
    /// the stack.
    pub fn activated_ability_object_id(
        &self,
        activated_ability_id: ActivatedAbilityId,
    ) -> Option<ObjectId> {
        self.activated_ability_object_ids.get(&activated_ability_id).copied()
    }

    /// Puts an activated ability on the stack.
    pub fn add_activated_ability_to_stack(
        &mut self,
        controller: PlayerName,
        activated_ability_id: ActivatedAbilityId,
    ) {
        let object_id = self.new_object_id();
        self.stack.push(StackItemState {
            id: StackItemId::ActivatedAbility(activated_ability_id),
            controller,
            targets: None,
            additional_costs_paid: StackCardAdditionalCostsPaid::None,
            modal_choice: None,
        });
        self.activated_ability_object_ids.insert(activated_ability_id, object_id);
    }

    /// Removes an activated ability from the stack.
    pub fn remove_activated_ability_from_stack(
        &mut self,
        activated_ability_id: ActivatedAbilityId,
    ) {
        if let Some((i, _)) = self
            .stack
            .iter()
            .enumerate()
            .rev()
            .find(|(_, item)| item.id == StackItemId::ActivatedAbility(activated_ability_id))
        {
            self.stack.remove(i);
        }
        self.activated_ability_object_ids.remove(&activated_ability_id);
    }

    fn new_object_id(&mut self) -> ObjectId {
        let result = self.next_object_id;
        // Hopefully we won't have more than 18,446,744,073,709,551,615 active
        // cards at once.
        self.next_object_id = ObjectId(result.0.wrapping_add(1));
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
                self.shuffled_into_decks.player_mut(controller).insert(DeckCardId(card_id));
            }
            Zone::Hand => {
                self.hands.player_mut(controller).insert(HandCardId(card_id));
            }
            Zone::Stack => {
                self.stack_card_set.player_mut(controller).insert(StackCardId(card_id));
                self.stack.push(StackItemState {
                    id: StackItemId::Card(StackCardId(card_id)),
                    controller,
                    targets: None,
                    additional_costs_paid: StackCardAdditionalCostsPaid::None,
                    modal_choice: None,
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
                self.shuffled_into_decks.player_mut(controller).remove(DeckCardId(card_id));
                let tops = self.tops_of_decks.player_mut(controller);
                if let Some(pos) = tops.iter().position(|&id| id == DeckCardId(card_id)) {
                    tops.remove(pos);
                }
            }
            Zone::Hand => {
                self.hands.player_mut(controller).remove(HandCardId(card_id));
            }
            Zone::Stack => {
                self.stack_card_set.player_mut(controller).remove(StackCardId(card_id));
                if let Some((i, _)) = self
                    .stack
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, item)| item.id == StackItemId::Card(StackCardId(card_id)))
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
