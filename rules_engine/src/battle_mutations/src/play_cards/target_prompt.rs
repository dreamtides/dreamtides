use ability_data::ability::Ability;
use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::StackCardId;
use battle_data::prompts::prompt_data::{Prompt, PromptContext, PromptData};
use battle_queries::predicate_queries::predicates;
use core_data::effect_source::EffectSource;

/// Adds a prompt to the `battle` for targets required to play the `card_id`
/// card.
pub fn add_target_prompt(
    battle: &mut BattleData,
    source: EffectSource,
    card_id: StackCardId,
) -> Option<()> {
    let card = battle.cards.card(card_id)?;
    let player = card.controller();

    for ability in &card.abilities {
        if let Ability::Event(effect) = ability {
            match effect {
                Effect::Effect(std_effect) => {
                    if let Some(target_predicate) = get_target_predicate(std_effect) {
                        let valid = predicates::matching_characters(
                            battle,
                            player,
                            source,
                            target_predicate.clone(),
                        );
                        battle.prompt = Some(PromptData {
                            player,
                            prompt: Prompt::ChooseCharacter { valid },
                            optional: false,
                            context: get_prompt_context(std_effect),
                        });

                        return Some(());
                    }
                }
                Effect::WithOptions(with_options) => {
                    if let Some(target_predicate) = get_target_predicate(&with_options.effect) {
                        let valid = predicates::matching_characters(
                            battle,
                            player,
                            source,
                            target_predicate.clone(),
                        );
                        battle.prompt = Some(PromptData {
                            player,
                            prompt: Prompt::ChooseCharacter { valid },
                            optional: with_options.is_optional(),
                            context: get_prompt_context(&with_options.effect),
                        });

                        return Some(());
                    }
                }
                Effect::List(_) => {
                    todo!("Handle multiple effects in an event ability");
                }
            }
        }
    }

    Some(())
}

/// Extracts the target predicate from a standard effect, if any.
fn get_target_predicate(effect: &StandardEffect) -> Option<Predicate> {
    match effect {
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => Some(target.clone()),
        StandardEffect::AbandonAtEndOfTurn { target } => Some(target.clone()),
        StandardEffect::BanishCharacter { target } => Some(target.clone()),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, .. } => Some(target.clone()),
        StandardEffect::BanishUntilNextMain { target } => Some(target.clone()),
        StandardEffect::BanishCollection { target, .. } => Some(target.clone()),
        StandardEffect::Copy { target } => Some(target.clone()),
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => Some(target.clone()),
        StandardEffect::DissolveCharacter { target } => Some(target.clone()),
        StandardEffect::DissolveCharactersCount { target, .. } => Some(target.clone()),
        StandardEffect::DissolveCharactersQuantity { target, .. } => Some(target.clone()),
        StandardEffect::GainControl { target } => Some(target.clone()),
        StandardEffect::GainsAegisThisTurn { target } => Some(target.clone()),
        StandardEffect::GainsReclaimUntilEndOfTurn { target, .. } => Some(target.clone()),
        StandardEffect::GainsSpark { target, .. } => Some(target.clone()),
        StandardEffect::GainsSparkForQuantity { target, .. } => Some(target.clone()),
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, .. } => Some(target.clone()),
        StandardEffect::MaterializeCharacter { target } => Some(target.clone()),
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => Some(target.clone()),
        StandardEffect::MaterializeSilentCopy { target, .. } => Some(target.clone()),
        StandardEffect::Negate { target } => Some(target.clone()),
        StandardEffect::PutOnTopOfEnemyDeck { target } => Some(target.clone()),
        StandardEffect::ReturnFromYourVoidToHand { target } => Some(target.clone()),
        StandardEffect::ReturnFromYourVoidToPlay { target } => Some(target.clone()),
        StandardEffect::ReturnToHand { target } => Some(target.clone()),
        _ => None,
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
