use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use battle_state::battle_cards::battlefield::Battlefield;
use core_data::card_types::{CardSubtype, CardType};
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;

use crate::battle_card_queries::card;

const NOCTURNE_STRUMMER: &str = "Nocturne Strummer";

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
    let stored = battle.cards.spark(controller, id)?;
    let Some(slot) = front_slot(battle, controller, id) else {
        return Some(stored);
    };
    Some(stored + supported_sentry_bonus(battle, controller, slot))
}

pub fn base_spark(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    card::get(battle, card_id).base_spark
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    card::get(battle, card_id).card_type
}

pub fn card_subtype(_battle: &BattleState, _card_id: impl CardIdType) -> Option<CardSubtype> {
    todo!("Implement card_subtype");
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    card::get(battle, card_id).is_fast
}

fn front_slot(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<usize> {
    battle.cards.battlefield(controller).front.iter().position(|slot| *slot == Some(id))
}

fn has_displayed_name(battle: &BattleState, id: CharacterId, expected: &str) -> bool {
    card::get_definition(battle, id).displayed_name == expected
}

fn supported_sentry_bonus(
    battle: &BattleState,
    controller: PlayerName,
    front_slot: usize,
) -> Spark {
    let battlefield = battle.cards.battlefield(controller);
    let supporters =
        Battlefield::supporting_back_slots(front_slot, battle.rules_config.front_row_size);
    let count = supporters
        .into_iter()
        .filter_map(|slot| battlefield.back.get(slot).copied().flatten())
        .filter(|id| has_displayed_name(battle, *id, NOCTURNE_STRUMMER))
        .count();
    Spark((count as u32) * 2)
}
