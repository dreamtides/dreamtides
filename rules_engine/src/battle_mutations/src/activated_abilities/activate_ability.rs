use ability_data::effect::ModelEffectChoiceIndex;
use battle_queries::battle_card_queries::card;
use battle_queries::battle_player_queries::costs;
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, CharacterId};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    ActivatedAbilityOption, PromptConfiguration, PromptData, PromptType,
};
use core_data::identifiers::AbilityNumber;
use core_data::types::PlayerName;

use crate::effects::pay_cost;
use crate::prompt_mutations::card_choice_prompts;

/// Activates an ability for a character by character ID. If ability_number is
/// provided, activates that specific ability. If ability_number is None and the
/// character has exactly one activated ability, it activates that ability
/// directly. If ability_number is None and the character has multiple activated
/// abilities, it creates a prompt for the player to choose which ability to
/// activate.
pub fn execute(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    ability_number: Option<AbilityNumber>,
) {
    battle_trace!("Activating ability for character", battle, player, character_id);

    if let Some(ability_num) = ability_number {
        // Specific ability number provided, activate it directly
        let activated_ability_id = ActivatedAbilityId { character_id, ability_number: ability_num };
        execute_internal(battle, player, activated_ability_id);
        return;
    }

    // No specific ability number provided, determine available abilities
    let abilities = card::ability_list(battle, character_id);
    let available_abilities: Vec<_> = abilities
        .activated_abilities
        .iter()
        .filter(|ability_data| {
            let activated_ability_id =
                ActivatedAbilityId { character_id, ability_number: ability_data.ability_number };

            let is_multi = ability_data
                .ability
                .options
                .as_ref()
                .map(|options| options.is_multi)
                .unwrap_or(false);

            // Check if this ability can be used (not already used this turn for non-multi
            // abilities)
            if !is_multi
                && battle
                    .activated_abilities
                    .player(player)
                    .activated_this_turn_cycle
                    .contains(&activated_ability_id)
            {
                return false;
            }

            // Check if ability is already on stack
            if battle.cards.activated_ability_object_id(activated_ability_id).is_some() {
                return false;
            }

            // Check if player can pay all costs
            ability_data.ability.costs.iter().all(|cost| costs::can_pay(battle, player, cost))
        })
        .collect();

    if available_abilities.is_empty() {
        panic_with!("No available activated abilities for character", battle, character_id);
    } else if available_abilities.len() == 1 {
        // Only one ability available, activate it directly
        let activated_ability_id = ActivatedAbilityId {
            character_id,
            ability_number: available_abilities[0].ability_number,
        };
        execute_internal(battle, player, activated_ability_id);
    } else {
        // Multiple abilities available, create a choice prompt
        let ability_options: Vec<ActivatedAbilityOption> = available_abilities
            .iter()
            .map(|ability_data| {
                let energy_cost = ability_data.ability.costs.iter().find_map(|cost| match cost {
                    ability_data::cost::Cost::Energy(energy) => Some(*energy),
                    _ => None,
                });

                ActivatedAbilityOption {
                    ability_number: ability_data.ability_number,
                    name: format!("Ability {}", ability_data.ability_number.0),
                    energy_cost,
                }
            })
            .collect();

        let source = EffectSource::Game { controller: player };
        battle.prompts.push_back(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseActivatedAbility {
                character_id,
                abilities: ability_options,
            },
            configuration: PromptConfiguration { optional: false },
        });

        battle_trace!("Created activated ability choice prompt", battle);
    }
}

/// Executes the selected activated ability from a character choice prompt.
pub fn execute_selected_ability(battle: &mut BattleState, player: PlayerName, choice_index: usize) {
    battle_trace!("Executing selected activated ability", battle, player, choice_index);

    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };

    let PromptType::ChooseActivatedAbility { character_id, abilities } = prompt.prompt_type else {
        panic_with!("Prompt is not an activated ability choice", battle);
    };

    let Some(selected_ability) = abilities.get(choice_index) else {
        panic_with!("Invalid ability choice index", battle, choice_index);
    };

    execute(battle, player, character_id, Some(selected_ability.ability_number));
}

/// Resumes adding prompts for an activated ability that was activated after an
/// initial prompt has been resolved.
///
/// This is used when an activated ability requires more than one sequential
/// choice.
pub fn resume_adding_activated_ability_prompts(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
    modal_choice: Option<ModelEffectChoiceIndex>,
) {
    card_choice_prompts::add_for_activated_ability(
        battle,
        player,
        activated_ability_id,
        modal_choice,
    );
}

/// Activates an ability of a character on the battlefield by putting it on the
/// stack.
fn execute_internal(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
) {
    battle_trace!("Activating ability", battle, player, activated_ability_id);
    let source = EffectSource::Activated { controller: player, activated_ability_id };

    let abilities = card::ability_list(battle, activated_ability_id.character_id);
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

    battle.push_animation(source, || BattleAnimation::ActivatedAbility {
        player,
        activated_ability_id,
    });

    resume_adding_activated_ability_prompts(battle, player, activated_ability_id, None);
}
