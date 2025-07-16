use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use battle_queries::battle_card_queries::card_abilities;
use battle_queries::battle_trace;
use battle_queries::card_ability_queries::effect_predicates;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, StackCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{
    PromptConfiguration, PromptData, PromptFor, PromptType,
};
use core_data::types::PlayerName;

/// Adds a prompt to the `battle` for targets required to play the `card_id`
/// card.
pub fn execute(battle: &mut BattleState, player: PlayerName, card_id: StackCardId) {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        let source = EffectSource::Event {
            controller: player,
            stack_card_id: card_id,
            ability_number: data.ability_number,
        };
        if let Some(prompt_data) = targeting_prompt(
            battle,
            player,
            source,
            &data.ability.effect,
            PromptFor::AddingItemToStack(card_id.into()),
        ) {
            battle_trace!("Adding target prompt", battle, prompt_data);
            battle.prompts.push_back(prompt_data);
            return;
        }
    }
}

/// Adds a prompt to the `battle` for targets required for an activated ability.
pub fn execute_for_activated_ability(
    battle: &mut BattleState,
    player: PlayerName,
    activated_ability_id: ActivatedAbilityId,
) {
    let abilities = card_abilities::query(battle, activated_ability_id.character_id);
    if let Some(ability_data) = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == activated_ability_id.ability_number)
    {
        let source = EffectSource::Activated { controller: player, activated_ability_id };
        if let Some(prompt_data) = targeting_prompt(
            battle,
            player,
            source,
            &ability_data.ability.effect,
            PromptFor::AddingItemToStack(activated_ability_id.into()),
        ) {
            battle_trace!("Adding target prompt for activated ability", battle, prompt_data);
            battle.prompts.push_back(prompt_data);
        }
    }
}

/// Creates a prompt data for an effect if it requires target selection.
/// Returns the prompt data if created, None otherwise.
fn targeting_prompt(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    effect: &Effect,
    prompt_for: PromptFor,
) -> Option<PromptData> {
    match effect {
        Effect::Effect(standard_effect) => standard_effect_targeting_prompt(
            battle,
            player,
            source,
            standard_effect,
            false,
            prompt_for,
        ),
        Effect::WithOptions(with_options) => standard_effect_targeting_prompt(
            battle,
            player,
            source,
            &with_options.effect,
            with_options.optional,
            prompt_for,
        ),
        Effect::List(effects) => {
            for effect_with_options in effects {
                if let Some(prompt_data) = standard_effect_targeting_prompt(
                    battle,
                    player,
                    source,
                    &effect_with_options.effect,
                    effect_with_options.optional,
                    prompt_for,
                ) {
                    return Some(prompt_data);
                }
            }
            None
        }
    }
}

/// Creates a prompt data for a standard effect if it requires target selection.
///
/// Returns the prompt data if targets are required and there are legal targets
/// available and None otherwise.
fn standard_effect_targeting_prompt(
    battle: &BattleState,
    player: PlayerName,
    source: EffectSource,
    effect: &StandardEffect,
    optional: bool,
    prompt_for: PromptFor,
) -> Option<PromptData> {
    if let Some(target_predicate) = effect_predicates::get_character_target_predicate(effect) {
        let valid = effect_predicates::matching_characters(battle, source, target_predicate);
        if valid.is_empty() {
            return None;
        }

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseCharacter { prompt_for, valid },
            configuration: PromptConfiguration { optional },
        })
    } else if let Some(target_predicate) = effect_predicates::get_stack_target_predicate(effect) {
        let valid = effect_predicates::matching_cards_on_stack(battle, source, target_predicate);
        if valid.is_empty() {
            return None;
        }

        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseStackCard { prompt_for, valid },
            configuration: PromptConfiguration { optional },
        })
    } else {
        None
    }
}
