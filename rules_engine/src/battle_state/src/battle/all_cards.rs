use std::rc::Rc;

use bit_set::BitSet;
use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use core_data::types::PlayerName;
use small_map::SmallMap;
use smallvec::SmallVec;

use crate::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};
use crate::battle::player_map::PlayerMap;
use crate::battle_cards::character_state::CharacterState;
use crate::battle_cards::stack_card_state::{StackCardState, StackCardTargets};
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    card_names: Rc<Vec<CardName>>,
    battlefield: PlayerMap<BitSet<usize>>,
    battlefield_state: PlayerMap<SmallMap<8, CharacterId, CharacterState>>,
    void: PlayerMap<BitSet<usize>>,
    hands: PlayerMap<BitSet<usize>>,
    decks: PlayerMap<BitSet<usize>>,
    stack: SmallVec<[StackCardState; 2]>,
    stack_set: PlayerMap<BitSet<usize>>,
    banished: PlayerMap<BitSet<usize>>,
}

impl AllCards {
    /// Returns the name of a card.
    ///
    /// Panics if no card with this ID exists.
    pub fn name(&self, id: impl CardIdType) -> CardName {
        self.card_names[id.card_id().0]
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

    /// Returns the set of characters on the battlefield for a given player
    pub fn battlefield(&self, player: PlayerName) -> &BitSet<usize> {
        self.battlefield.player(player)
    }

    /// Returns the state of characters on the battlefield for a given player
    pub fn battlefield_state(
        &self,
        player: PlayerName,
    ) -> &SmallMap<8, CharacterId, CharacterState> {
        self.battlefield_state.player(player)
    }

    /// Returns true if a stack is currently active.
    pub fn has_stack(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Returns the top card on the stack, if any.
    pub fn top_of_stack(&self) -> Option<&StackCardState> {
        self.stack.last()
    }

    /// Returns the set of cards on the stack for a given player.
    pub fn stack_set(&self, player: PlayerName) -> &BitSet<usize> {
        self.stack_set.player(player)
    }

    /// Returns all currently known Card IDs in an undefined order
    pub fn all_cards(&self) -> impl Iterator<Item = CardId> + '_ {
        self.card_names.iter().enumerate().map(|(i, _)| CardId(i))
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
    }

    /// Returns true if the indicated card is present in the indicated zone.
    pub fn contains_card(&self, controller: PlayerName, card_id: CardId, zone: Zone) -> bool {
        match zone {
            Zone::Banished => self.banished.player(controller).contains(card_id.0),
            Zone::Battlefield => self.battlefield.player(controller).contains(card_id.0),
            Zone::Deck => self.decks.player(controller).contains(card_id.0),
            Zone::Hand => self.hands.player(controller).contains(card_id.0),
            Zone::Stack => self.stack_set.player(controller).contains(card_id.0),
            Zone::Void => self.void.player(controller).contains(card_id.0),
        }
    }

    fn add_to_zone(&mut self, controller: PlayerName, card_id: CardId, zone: Zone) {
        match zone {
            Zone::Banished => {
                self.banished.player_mut(controller).insert(card_id.0);
            }
            Zone::Battlefield => {
                self.battlefield.player_mut(controller).insert(card_id.0);
                self.battlefield_state
                    .player_mut(controller)
                    .insert(CharacterId(card_id), CharacterState::default());
            }
            Zone::Deck => {
                self.decks.player_mut(controller).insert(card_id.0);
            }
            Zone::Hand => {
                self.hands.player_mut(controller).insert(card_id.0);
            }
            Zone::Stack => {
                self.stack_set.player_mut(controller).insert(card_id.0);
                self.stack.push(StackCardState {
                    id: StackCardId(card_id),
                    controller,
                    targets: StackCardTargets::None,
                });
            }
            Zone::Void => {
                self.void.player_mut(controller).insert(card_id.0);
            }
        }
    }

    fn remove_from_zone(&mut self, controller: PlayerName, card_id: CardId, zone: Zone) {
        match zone {
            Zone::Banished => {
                self.banished.player_mut(controller).remove(card_id.0);
            }
            Zone::Battlefield => {
                self.battlefield.player_mut(controller).remove(card_id.0);
                self.battlefield_state.player_mut(controller).remove(&CharacterId(card_id));
            }
            Zone::Deck => {
                self.decks.player_mut(controller).remove(card_id.0);
            }
            Zone::Hand => {
                self.hands.player_mut(controller).remove(card_id.0);
            }
            Zone::Stack => {
                self.stack_set.player_mut(controller).remove(card_id.0);
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
                self.void.player_mut(controller).remove(card_id.0);
            }
        }
    }
}
