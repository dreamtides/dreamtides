use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use core_data::card_types::{CardSubtype, CardType};
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;

use crate::battle_card_queries::card;

/// Returns the energy cost of a card, or 0 if it has no energy cost.
///
/// Cards may not have an energy cost due to their card type (e.g. dreamwell
/// cards) or may not have a cost due to their ability (e.g. modal cards).
#[inline(always)]
pub fn converted_energy_cost(battle: &BattleState, card_id: impl CardIdType) -> Energy {
    base_energy_cost(battle, card_id).unwrap_or_default()
}

#[inline(always)]
pub fn base_energy_cost(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    card::get(battle, card_id).base_energy_cost
}

/// Returns the player who currently controls a given card.
pub fn controller(battle: &BattleState, card_id: impl CardIdType) -> PlayerName {
    card::get(battle, card_id).owner
}

pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    battle.cards.spark(controller, id)
}

pub fn base_spark(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    card::get(battle, card_id).base_spark
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    card::get(battle, card_id).card_type
}

pub fn card_subtype(_battle: &BattleState, _card_id: impl CardIdType) -> Option<CardSubtype> {
    Some(CardSubtype::Musician)
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    card::get(battle, card_id).is_fast
}
