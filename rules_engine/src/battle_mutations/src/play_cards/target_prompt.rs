use ability_data::ability::Ability;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use assert_with::assert_that;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::StackCardId;
use battle_data::prompts::prompt_data::{Prompt, PromptContext, PromptData};
use battle_queries::predicate_queries::predicates;
use core_data::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing::info;

/// Adds a prompt to the `battle` for targets required to play the `card_id`
/// card.
pub fn add_target_prompt(battle: &mut BattleData, source: EffectSource, card_id: StackCardId) {
    let Some(card) = battle.cards.card(card_id) else {
        return;
    };
    let player = card.controller();

    for ability in &card.abilities {
        if let Ability::Event(effect) = ability {
            if let Some(prompt_data) = create_prompt_for_effect(battle, player, source, effect) {
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
            create_prompt_for_standard_effect(battle, player, source, standard_effect, false)
        }
        Effect::WithOptions(with_options) => create_prompt_for_standard_effect(
            battle,
            player,
            source,
            &with_options.effect,
            with_options.is_optional(),
        ),
        Effect::List(effects) => {
            for effect_with_options in effects {
                if let Some(prompt_data) = create_prompt_for_standard_effect(
                    battle,
                    player,
                    source,
                    &effect_with_options.effect,
                    effect_with_options.is_optional(),
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
fn create_prompt_for_standard_effect(
    battle: &BattleData,
    player: PlayerName,
    source: EffectSource,
    std_effect: &StandardEffect,
    optional: bool,
) -> Option<PromptData> {
    if let Some(target_predicate) = predicates::get_character_target_predicate(std_effect) {
        let valid =
            predicates::matching_characters(battle, player, source, target_predicate.clone());
        assert_that!(!valid.is_empty(), battle, || format!(
            "No valid characters for {:?}",
            std_effect
        ));
        info!("Adding prompt for characters {:?}", valid);
        Some(PromptData {
            player,
            prompt: Prompt::ChooseCharacter { valid },
            optional,
            context: get_prompt_context(std_effect),
        })
    } else if let Some(target_predicate) = predicates::get_stack_target_predicate(std_effect) {
        let valid =
            predicates::matching_cards_on_stack(battle, player, source, target_predicate.clone());
        assert_that!(!valid.is_empty(), battle, || format!(
            "No valid stack cards for {:?}",
            std_effect
        ));
        Some(PromptData {
            player,
            prompt: Prompt::ChooseStackCard { valid },
            optional,
            context: get_prompt_context(std_effect),
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
