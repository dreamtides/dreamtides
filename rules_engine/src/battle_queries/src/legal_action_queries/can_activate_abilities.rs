use ability_data::cost::Cost;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, CharacterId};
use battle_state::battle_cards::card_set::CardSet;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::card;
use crate::legal_action_queries::can_play_cards::FastOnly;
use crate::legal_action_queries::legal_actions_cache;

/// Returns the set of characters controlled by a player that have activated
/// abilities based on their own internal state & costs. If `fast_only` is set,
/// only abilities with the `fast` property are returned.
///
/// This does *not* check whether it is legal to activate abilities in the
/// larger current battle state, e.g. whether it is the player's turn.
pub fn for_player(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> CardSet<CharacterId> {
    let mut candidates =
        legal_actions_cache::activate_ability_candidates(battle, player, fast_only);
    candidates.intersect_with(battle.cards.battlefield(player));

    let mut characters_with_abilities = CardSet::new();

    for character_id in &candidates {
        let abilities = card::ability_list(battle, character_id);
        let mut has_available_ability = false;

        for ability_data in &abilities.activated_abilities {
            let activated_ability_id =
                ActivatedAbilityId { character_id, ability_number: ability_data.ability_number };
            let is_multi = ability_data
                .ability
                .options
                .as_ref()
                .map(|options| options.is_multi)
                .unwrap_or(false);

            if !is_multi
                && battle
                    .activated_abilities
                    .player(player)
                    .activated_this_turn_cycle
                    .contains(&activated_ability_id)
            {
                continue;
            }

            if battle.cards.activated_ability_object_id(activated_ability_id).is_some() {
                continue;
            }

            // This character has at least one available ability
            has_available_ability = true;
            break;
        }

        if has_available_ability {
            characters_with_abilities.insert(character_id);
        }
    }

    characters_with_abilities
}

/// Returns the lowest energy cost to activate any of a character's activated
/// abilities, or None if this character does not have activated abilities.
///
/// Returns 0 if this character has an activated ability which does not require
/// an energy cost.
///
/// If `fast_only` is provided, only considers fast abilities.
pub fn activated_ability_energy_cost(
    battle: &BattleState,
    character: CharacterId,
    fast_only: FastOnly,
) -> Option<Energy> {
    let abilities = card::ability_list(battle, character);

    if abilities.activated_abilities.is_empty() {
        return None;
    }

    let mut has_ability_with_no_energy_cost = false;
    let mut min_energy_cost: Option<Energy> = None;

    for ability_data in &abilities.activated_abilities {
        let options = ability_data.ability.options.as_ref();
        if fast_only == FastOnly::Yes {
            let is_fast = options.map(|options| options.is_fast).unwrap_or(false);
            if !is_fast {
                continue;
            }
        }

        let energy_cost = ability_data.ability.costs.iter().find_map(|cost| match cost {
            Cost::Energy(energy) => Some(*energy),
            _ => None,
        });

        match energy_cost {
            Some(cost) => {
                min_energy_cost = Some(match min_energy_cost {
                    Some(current_min) => current_min.min(cost),
                    None => cost,
                });
            }
            None => {
                has_ability_with_no_energy_cost = true;
            }
        }
    }

    if has_ability_with_no_energy_cost { Some(Energy(0)) } else { min_energy_cost }
}
