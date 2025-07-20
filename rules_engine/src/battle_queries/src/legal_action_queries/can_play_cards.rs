use std::hash::Hash;

use ability_data::static_ability::StandardStaticAbility;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{AbilityId, CardId, CardIdType, HandCardId, VoidCardId};
use battle_state::battle_cards::ability_list::{AbilityReference, CanPlayRestriction};
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::{card, card_abilities, card_properties};
use crate::legal_action_queries::{has_legal_additional_costs, has_legal_targets};
use crate::panic_with;

/// Whether only cards with the `fast` property should be returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastOnly {
    Yes,
    No,
}

/// Returns the set of cards in a player's hand that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_hand(battle: &BattleState, player: PlayerName, fast_only: FastOnly) -> Vec<HandCardId> {
    // Tested using a bitset here, but Vec is consistently faster (4% overall
    // benchmark improvement) possibly due to better random element selection.
    let mut legal_cards = Vec::new();

    for card_id in battle.cards.hand(player) {
        // Quick check for energy cost first, since this is the most common
        // reason for a card not being playable.
        let energy_cost = card_properties::converted_energy_cost(battle, card_id);
        if energy_cost > battle.players.player(player).current_energy {
            continue;
        }

        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, card_id) {
            continue;
        }

        add_if_meets_restriction(battle, player, card_id, energy_cost, &mut legal_cards);
    }

    legal_cards
}

/// A card that can be played from the void via a given ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CanPlayFromVoid {
    pub card_id: VoidCardId,
    pub via_ability_id: AbilityId,
}

impl From<CanPlayFromVoid> for CardId {
    fn from(value: CanPlayFromVoid) -> Self {
        value.card_id.card_id()
    }
}

/// Returns the set of cards in a player's void that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_void(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> Vec<CanPlayFromVoid> {
    let mut legal_cards = Vec::new();

    for card_id in battle.ability_state.has_play_from_void_ability.player(player).iter() {
        let Some(from_void_with_cost) = can_play_from_void_energy_cost(battle, card_id) else {
            continue;
        };

        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, card_id) {
            continue;
        }

        if from_void_with_cost.cost > battle.players.player(player).current_energy {
            continue;
        }

        let can_play_from_void =
            CanPlayFromVoid { card_id, via_ability_id: from_void_with_cost.via_ability_id };

        add_if_meets_restriction(
            battle,
            player,
            can_play_from_void,
            from_void_with_cost.cost,
            &mut legal_cards,
        );
    }

    legal_cards
}

/// Returns the energy cost of playing a card from the void with a given
/// ability.
///
/// Panics if the card cannot be played from the void or has no energy cost.
pub fn play_from_void_energy_cost(
    battle: &BattleState,
    card_id: VoidCardId,
    ability_id: AbilityId,
) -> Energy {
    let ability = card_abilities::ability(battle, ability_id);
    let cost = if let AbilityReference::Static(static_ability) = ability {
        can_play_from_void_with_static_ability(
            battle,
            card_id,
            ability_id.ability_number,
            static_ability.standard_static_ability(),
        )
        .map(|cost| cost.cost)
    } else {
        None
    };

    if let Some(cost) = cost {
        cost
    } else {
        panic_with!("Card has no void energy cost", battle, card_id, ability_id);
    }
}

/// Cost and ability ID of a card that can be played from the void.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FromVoidWithCost {
    pub cost: Energy,
    pub via_ability_id: AbilityId,
}

/// Check if a card can be played from the void, and returns the energy cost
/// of playing it if it can be. If there are multiple abilities that can be
/// used to play the card, returns the one with the lowest cost.
///
/// Other costs are handled in the 'meets_restriction' step.
fn can_play_from_void_energy_cost(
    battle: &BattleState,
    card_id: VoidCardId,
) -> Option<FromVoidWithCost> {
    card_abilities::query(battle, card_id)
        .static_abilities
        .iter()
        .filter_map(|ability_data| {
            can_play_from_void_with_static_ability(
                battle,
                card_id,
                ability_data.ability_number,
                ability_data.ability.standard_static_ability(),
            )
        })
        .min()
}

/// Returns the energy cost of playing a card from the void with a given static
/// ability, if it can be played.
fn can_play_from_void_with_static_ability(
    battle: &BattleState,
    card_id: VoidCardId,
    ability_number: AbilityNumber,
    ability: &StandardStaticAbility,
) -> Option<FromVoidWithCost> {
    let ability_id = AbilityId { card_id: card_id.card_id(), ability_number };
    match ability {
        StandardStaticAbility::PlayFromVoid(play) => {
            let cost = match play.energy_cost {
                Some(energy_cost) => energy_cost,
                // Cards with no energy cost (e.g. modal cards) are treated as
                // having a cost of 0
                None => card_properties::converted_energy_cost(battle, card_id),
            };
            Some(FromVoidWithCost { cost, via_ability_id: ability_id })
        }
        StandardStaticAbility::PlayOnlyFromVoid => {
            let cost = card_properties::converted_energy_cost(battle, card_id);
            Some(FromVoidWithCost { cost, via_ability_id: ability_id })
        }
        _ => None,
    }
}

/// Adds a card to the list of legal cards if it meets the restriction.
fn add_if_meets_restriction<TId: Into<CardId> + Copy>(
    battle: &BattleState,
    player: PlayerName,
    to_add: TId,
    cost: Energy,
    legal_cards: &mut Vec<TId>,
) {
    let card_id = to_add.into();

    let meets = match card::get(battle, card_id).can_play_restriction {
        Some(restriction) => meets_restriction(battle, player, restriction, cost),
        None => {
            // No fast version of the 'can play' restriction, check all card
            // abilities.
            has_legal_targets::for_event(battle, player, card_id.card_id())
                && has_legal_additional_costs::for_event(battle, player, card_id.card_id(), cost)
        }
    };

    if meets {
        legal_cards.push(to_add);
    }
}

fn meets_restriction(
    battle: &BattleState,
    controller: PlayerName,
    restriction: CanPlayRestriction,
    energy_cost: Energy,
) -> bool {
    match restriction {
        CanPlayRestriction::Unrestricted => true,
        CanPlayRestriction::EnemyCharacter => {
            !battle.cards.battlefield(controller.opponent()).is_empty()
        }
        CanPlayRestriction::EnemyStackCard => {
            !battle.cards.stack_set(controller.opponent()).is_empty()
        }
        CanPlayRestriction::EnemyStackCardOfType(card_type) => battle
            .cards
            .stack_set(controller.opponent())
            .iter()
            .any(|id| card_properties::card_type(battle, id) == card_type),
        CanPlayRestriction::AdditionalEnergyAvailable(required_energy) => {
            battle.players.player(controller).current_energy - energy_cost >= required_energy
        }
    }
}
