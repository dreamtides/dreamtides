use ability_data::effect::{Effect, EffectWithOptions};
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use assert_with::assert_that;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_data::TargetId;
use battle_data::battle_cards::card_id::{CharacterId, StackCardId};
use battle_data::battle_cards::zone::Zone;
use battle_data::prompt_types::prompt_data::{
    Prompt, PromptChoice, PromptConfiguration, PromptContext, PromptData,
};
use battle_queries::cost_queries::costs;
use battle_queries::predicate_queries::effect_predicates;

use crate::character_mutations::dissolve;
use crate::core::prompts;
use crate::effects::{negate, pay_cost};

/// Applies an effect to the battle state.
pub fn apply(
    battle: &mut BattleData,
    source: EffectSource,
    effect: Effect,
    targets: Vec<TargetId>,
) -> Option<()> {
    match effect {
        Effect::Effect(standard_effect) => {
            apply_standard_effect(battle, source, standard_effect, &targets)
        }
        Effect::WithOptions(effect_with_options) => {
            apply_effect_with_options(battle, source, effect_with_options, &targets)
        }
        Effect::List(effects) => apply_list_effect(battle, source, effects, &targets),
    }
}

fn apply_effect_with_options(
    _battle: &mut BattleData,
    _source: EffectSource,
    _effect: EffectWithOptions,
    _targets: &[TargetId],
) -> Option<()> {
    todo!("Implement effect with options")
}

fn apply_list_effect(
    battle: &mut BattleData,
    source: EffectSource,
    effects: Vec<EffectWithOptions>,
    targets: &[TargetId],
) -> Option<()> {
    for effect in effects {
        apply_effect_with_options(battle, source, effect, targets);
    }
    Some(())
}

fn apply_standard_effect(
    battle: &mut BattleData,
    source: EffectSource,
    effect: StandardEffect,
    targets: &[TargetId],
) -> Option<()> {
    if effect_predicates::has_targets(&effect) {
        assert_that!(!targets.is_empty(), battle, || format!(
            "Effect {:?} requires targets",
            effect
        ));
    }
    match effect {
        StandardEffect::DissolveCharacter { .. } => {
            for character_id in character_ids(targets) {
                dissolve::apply(battle, source, character_id);
            }
        }
        StandardEffect::Negate { .. } => negate(battle, source, targets),
        StandardEffect::NegateUnlessPaysCost { cost, .. } => {
            if costs::can_pay(battle, source.controller().opponent(), &cost) {
                prompts::set(battle, PromptData {
                    source,
                    player: source.controller().opponent(),
                    prompt: Prompt::Choose {
                        choices: vec![
                            PromptChoice {
                                label: "Pay $2".to_string(),
                                effect: Effect::Effect(StandardEffect::PayCost { cost }),
                                targets: vec![],
                            },
                            PromptChoice {
                                label: "Decline".to_string(),
                                effect: Effect::Effect(StandardEffect::Negate {
                                    target: Predicate::It,
                                }),
                                targets: targets.to_vec(),
                            },
                        ],
                    },
                    context: PromptContext::TargetNegativeEffect,
                    configuration: PromptConfiguration {
                        move_source_to: Some(Zone::Void),
                        ..Default::default()
                    },
                });
            } else {
                negate(battle, source, targets);
            }
        }
        StandardEffect::PayCost { cost } => pay_cost::apply(battle, source, cost),
        _ => todo!("Implement {:?}", effect),
    }
    Some(())
}

fn negate(battle: &mut BattleData, source: EffectSource, targets: &[TargetId]) {
    for stack_card_id in stack_card_ids(targets) {
        negate::apply(battle, source, stack_card_id);
    }
}

fn character_ids(targets: &[TargetId]) -> impl Iterator<Item = CharacterId> + '_ {
    targets.iter().filter_map(|target| match target {
        TargetId::Character(character_id) => Some(*character_id),
        _ => None,
    })
}

fn stack_card_ids(targets: &[TargetId]) -> impl Iterator<Item = StackCardId> + '_ {
    targets.iter().filter_map(|target| match target {
        TargetId::StackCard(stack_card_id) => Some(*stack_card_id),
        _ => None,
    })
}
