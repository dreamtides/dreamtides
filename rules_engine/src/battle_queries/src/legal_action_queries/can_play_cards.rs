use ability_data::static_ability::StandardStaticAbility;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, HandCardId, VoidCardId};
use battle_state::battle_cards::ability_list::CanPlayRestriction;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::{card, card_abilities, card_properties};
use crate::legal_action_queries::{has_legal_additional_costs, has_legal_targets};

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
        let Some(energy_cost) = card_properties::energy_cost(battle, card_id) else {
            continue;
        };

        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, card_id) {
            continue;
        }

        if energy_cost > battle.players.player(player).current_energy {
            continue;
        }

        add_if_meets_restriction(battle, player, card_id, energy_cost, &mut legal_cards);
    }

    legal_cards
}

/// Returns the set of cards in a player's void that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_void(battle: &BattleState, player: PlayerName, fast_only: FastOnly) -> Vec<VoidCardId> {
    let mut legal_cards = Vec::new();

    for card_id in battle.static_abilities.player(player).has_play_from_void_ability.iter() {
        let Some(energy_cost) = can_play_from_void_for_energy_cost(battle, card_id) else {
            continue;
        };

        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, card_id) {
            continue;
        }

        if energy_cost > battle.players.player(player).current_energy {
            continue;
        }

        add_if_meets_restriction(battle, player, card_id, energy_cost, &mut legal_cards);
    }

    legal_cards
}

/// Check if a card can be played from the void, and returns the energy cost
/// of playing it if it can.
///
/// Other costs are handled in the 'meets_restriction' step.
fn can_play_from_void_for_energy_cost(battle: &BattleState, card_id: VoidCardId) -> Option<Energy> {
    card_abilities::query(battle, card_id)
        .static_abilities
        .iter()
        .filter_map(|ability_data| {
            can_play_from_void_with_static_ability(
                battle,
                card_id,
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
    ability: &StandardStaticAbility,
) -> Option<Energy> {
    match ability {
        StandardStaticAbility::PlayFromVoid(play) => match play.energy_cost {
            Some(energy_cost) => Some(energy_cost),
            None => card_properties::energy_cost(battle, card_id),
        },
        StandardStaticAbility::PlayOnlyFromVoid => card_properties::energy_cost(battle, card_id),
        _ => None,
    }
}

/// Adds a card to the list of legal cards if it meets the restriction.
fn add_if_meets_restriction<TId: CardIdType>(
    battle: &BattleState,
    player: PlayerName,
    card_id: TId,
    cost: Energy,
    legal_cards: &mut Vec<TId>,
) {
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
        legal_cards.push(card_id);
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
