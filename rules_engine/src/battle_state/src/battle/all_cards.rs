use bit_set::BitSet;
use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};
use crate::battle::player_map::PlayerMap;
use crate::battle_cards::character_state::CharacterState;
use crate::battle_cards::stack_card_state::StackCardState;
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug, Default)]
pub struct AllCards {
    card_names: Vec<CardName>,
    character_states: PlayerMap<Vec<CharacterState>>,
    battlefield: PlayerMap<BitSet>,
    void: PlayerMap<BitSet>,
    hands: PlayerMap<BitSet>,
    decks: PlayerMap<BitSet>,
    stack: Vec<StackCardState>,
    banished: PlayerMap<BitSet>,
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
    /// Returns None if this character is not present on the battlefield or has
    /// no Spark value.
    pub fn spark(&self, controller: PlayerName, id: CharacterId) -> Option<Spark> {
        self.character_states
            .player(controller)
            .iter()
            .find(|character_state| character_state.id == id)
            .and_then(|character_state| character_state.spark)
    }

    /// Returns the top card on the stack, if any.
    pub fn top_of_stack(&self) -> Option<&StackCardState> {
        self.stack.last()
    }

    /// Returns all currently known Card IDs in an undefined order
    pub fn all_cards(&self) -> impl Iterator<Item = CardId> + '_ {
        self.card_names.iter().enumerate().map(|(i, _)| CardId(i))
    }

    /// Moves a card from its current zone to a new zone, if it is present.
    /// Generally you should use the `move_card` module instead of invoking this
    /// directly.
    ///
    /// Panics if the indicated card is not found in the 'from' zone.
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
            Zone::Stack => {
                self.stack.iter().any(|stack_card| stack_card.id == StackCardId(card_id))
            }
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
            }
            Zone::Deck => {
                self.decks.player_mut(controller).insert(card_id.0);
            }
            Zone::Hand => {
                self.hands.player_mut(controller).insert(card_id.0);
            }
            Zone::Stack => {
                self.stack.push(StackCardState {
                    id: StackCardId(card_id),
                    controller,
                    targets: BitSet::new(),
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
            }
            Zone::Deck => {
                self.decks.player_mut(controller).remove(card_id.0);
            }
            Zone::Hand => {
                self.hands.player_mut(controller).remove(card_id.0);
            }
            Zone::Stack => {
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
