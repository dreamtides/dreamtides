use ability_data::ability::Ability;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use assert_with::assert_that;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle_cards::card_id::StackCardId;
use battle_data_old::prompt_types::prompt_data::{
    PromptConfiguration, PromptContext, PromptData, PromptType,
};
use battle_queries_old::predicate_queries::effect_predicates;
use core_data::types::PlayerName;
use logging::battle_trace;

/// Adds a prompt to the `battle` for targets required to play the `card_id`
/// card.
pub fn add_target_prompt(battle: &mut BattleData, source: EffectSource, card_id: StackCardId) {
    let Some(card) = battle.cards.card(card_id) else {
        return;
    };
    let player = card.controller();

    for ability in &card.abilities {
        if let Ability::Event(event) = ability {
            if let Some(prompt_data) =
                create_prompt_for_effect(battle, player, source, &event.effect)
            {
                battle_trace!("Adding target prompt", battle, prompt_data);
                battle.prompt = Some(prompt_data);
                return;
            }
        }
    }
}

/// Creates a prompt data for an effect if it requires target selection.
/// Returns the prompt data if created, None otherwise.
fn create_prompt_for_effect(
    battle: &BattleData,
    player: PlayerName,
    source: EffectSource,
    effect: &Effect,
) -> Option<PromptData> {
    match effect {
        Effect::Effect(standard_effect) => {
            create_prompt_for_targeting(battle, player, source, standard_effect, false)
        }
        Effect::WithOptions(with_options) => create_prompt_for_targeting(
            battle,
            player,
            source,
            &with_options.effect,
            with_options.optional,
        ),
        Effect::List(effects) => {
            for effect_with_options in effects {
                if let Some(prompt_data) = create_prompt_for_targeting(
                    battle,
                    player,
                    source,
                    &effect_with_options.effect,
                    effect_with_options.optional,
                ) {
                    return Some(prompt_data);
                }
            }
            None
        }
    }
}

/// Creates a prompt data for a standard effect if it requires target selection.
/// Returns the prompt data if created, None otherwise.
fn create_prompt_for_targeting(
    battle: &BattleData,
    player: PlayerName,
    source: EffectSource,
    std_effect: &StandardEffect,
    optional: bool,
) -> Option<PromptData> {
    if let Some(target_predicate) = effect_predicates::get_character_target_predicate(std_effect) {
        let valid =
            effect_predicates::matching_characters(battle, source, target_predicate.clone());
        assert_that!(!valid.is_empty(), battle, || format!(
            "No valid characters for {:?}",
            std_effect
        ));
        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseCharacter { valid },
            context: get_prompt_context(std_effect),
            configuration: PromptConfiguration { optional, ..Default::default() },
        })
    } else if let Some(target_predicate) = effect_predicates::get_stack_target_predicate(std_effect)
    {
        let valid =
            effect_predicates::matching_cards_on_stack(battle, source, target_predicate.clone());
        assert_that!(!valid.is_empty(), battle, || format!(
            "No valid stack cards for {:?}",
            std_effect
        ));
        Some(PromptData {
            source,
            player,
            prompt_type: PromptType::ChooseStackCard { valid },
            context: get_prompt_context(std_effect),
            configuration: PromptConfiguration { optional, ..Default::default() },
        })
    } else {
        None
    }
}

/// Determines whether an effect is positive or negative for the target.
fn get_prompt_context(effect: &StandardEffect) -> PromptContext {
    match effect {
        StandardEffect::DissolveCharacter { .. }
        | StandardEffect::DissolveCharactersCount { .. }
        | StandardEffect::DissolveCharactersQuantity { .. }
        | StandardEffect::BanishCharacter { .. }
        | StandardEffect::BanishCharacterUntilLeavesPlay { .. }
        | StandardEffect::BanishUntilNextMain { .. }
        | StandardEffect::BanishCollection { .. }
        | StandardEffect::Negate { .. }
        | StandardEffect::PutOnTopOfEnemyDeck { .. }
        | StandardEffect::AbandonAtEndOfTurn { .. } => PromptContext::TargetNegativeEffect,
        _ => PromptContext::TargetPositiveEffect,
    }
}
