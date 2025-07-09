use battle_queries::battle_card_queries::card_abilities;
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::ActivatedAbilityId;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::effects::pay_cost;
use crate::prompt_mutations::add_targeting_prompt;

/// Activates an ability of a character on the battlefield by putting it on the
/// stack.
pub fn execute(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
) {
    battle_trace!("Activating ability", battle, player, activated_ability_id);
    let source = EffectSource::Activated { controller: player, activated_ability_id };

    let abilities = card_abilities::query(battle, activated_ability_id.character_id);
    let Some(ability_data) = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == activated_ability_id.ability_number)
    else {
        panic_with!("Activated ability not found", battle, activated_ability_id);
    };

    battle
        .activated_abilities
        .player_mut(player)
        .activated_this_turn_cycle
        .insert(activated_ability_id);

    for cost in &ability_data.ability.costs {
        pay_cost::execute(battle, source, player, cost);
    }

    battle.cards.add_activated_ability_to_stack(player, activated_ability_id);

    battle.stack_priority = Some(player.opponent());
    add_targeting_prompt::execute_for_activated_ability(battle, player, activated_ability_id);

    battle.push_animation(source, || BattleAnimation::ActivatedAbility {
        player,
        activated_ability_id,
    });
}
