use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::ActivatedAbilityId;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_abilities;
use crate::battle_player_queries::costs;
use crate::legal_action_queries::can_play_cards::FastOnly;

/// Returns the set of abilities that are activatable by a player based on their
/// own internal state & costs. If `fast_only` is set, only abilities with the
/// `fast` property are returned.
///
/// This does *not* check whether it is legal to activate abilities in the
/// larger current battle state, e.g. whether it is the player's turn.
pub fn for_player(
    battle: &BattleState,
    player: PlayerName,
    fast_only: FastOnly,
) -> Vec<ActivatedAbilityId> {
    let mut activatable_abilities = Vec::new();

    for character_id in battle.activated_abilities.player(player).characters.iter() {
        let abilities = card_abilities::query(battle, character_id);

        for ability_data in &abilities.activated_abilities {
            let options = ability_data.ability.options.as_ref();
            if fast_only == FastOnly::Yes {
                let is_fast = options.map(|options| options.is_fast).unwrap_or(false);
                if !is_fast {
                    continue;
                }
            }

            let can_pay_all_costs =
                ability_data.ability.costs.iter().all(|cost| costs::can_pay(battle, player, cost));
            if !can_pay_all_costs {
                continue;
            }

            let activated_ability_id =
                ActivatedAbilityId { character_id, ability_number: ability_data.ability_number };
            let is_multi = options.map(|options| options.is_multi).unwrap_or(false);

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

            // TODO: Check targeting restrictions.

            activatable_abilities.push(activated_ability_id);
        }
    }

    activatable_abilities
}
