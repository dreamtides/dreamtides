use std::sync::Arc;

use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::ability_list::AbilityList;
use battle_state::battle_cards::battle_card_state::BattleCardState;
use core_data::identifiers::BaseCardId;
use tabula_data::card_definition::CardDefinition;

use crate::battle_card_queries::card;

/// Returns the state of a card.
///
/// Panics if the card ID is invalid.
#[inline(always)]
pub fn get(battle: &BattleState, card_id: impl CardIdType) -> &BattleCardState {
    &battle.cards[card_id]
}

/// Returns a mutable reference to the state of a card.
///
/// Panics if the card ID is invalid.
#[inline(always)]
pub fn get_mut(battle: &mut BattleState, card_id: impl CardIdType) -> &mut BattleCardState {
    &mut battle.cards[card_id]
}

/// Returns the definition of a card.
pub fn get_definition(battle: &BattleState, card_id: impl CardIdType) -> Arc<CardDefinition> {
    let identity = card::get(battle, card_id).identity;
    battle.card_definitions.get_definition(identity)
}

/// Returns the [AbilityList] for a given card ID.
pub fn ability_list(battle: &BattleState, card_id: impl CardIdType) -> Arc<AbilityList> {
    let identity = card::get(battle, card_id).identity;
    battle.card_definitions.get_ability_list(identity)
}

/// Returns the base card ID for a given card ID.
pub fn get_base_card_id(battle: &BattleState, card_id: impl CardIdType) -> BaseCardId {
    get_definition(battle, card_id).base_card_id
}
