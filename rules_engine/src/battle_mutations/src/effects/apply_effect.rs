use std::iter;

use ability_data::ability::EventAbility;
use ability_data::effect::{Effect, EffectWithOptions};
use battle_queries::assert_that;
use battle_queries::battle_card_queries::stack_card_queries;
use battle_state::battle::battle_state::{BattleState, PendingEffect};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::card_id::StackCardId;
use battle_state::battle_cards::ability_list::AbilityData;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use either::Either;

use crate::effects::apply_standard_effect;

/// Marker struct indicating that an effect was applied to the battle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectWasApplied;

/// Applies all effects for the given [EventAbility] list to the [BattleState].
pub fn execute_event_abilities(
    battle: &mut BattleState,
    controller: PlayerName,
    stack_card_id: StackCardId,
    abilities: &[AbilityData<EventAbility>],
    requested_targets: Option<&EffectTargets>,
) {
    match abilities {
        [] => {}
        [ability] => {
            let source = EffectSource::Event {
                controller,
                stack_card_id,
                ability_number: ability.ability_number,
            };
            execute(battle, source, &ability.ability.effect, requested_targets);
        }
        _ => {
            assert_that!(
                battle.pending_effects.is_empty(),
                "Pending effects must be empty",
                battle
            );
            let targets = stack_card_queries::validate_targets(battle, requested_targets);
            battle.pending_effects = abilities
                .iter()
                .flat_map(|ability_data| {
                    let source = EffectSource::Event {
                        controller,
                        stack_card_id,
                        ability_number: ability_data.ability_number,
                    };
                    flatten_effect(source, &ability_data.ability.effect, targets.cloned())
                })
                .collect();
            execute_pending_effects_if_no_active_prompt(battle);
        }
    }
}

/// Applies an effect to the given [BattleState]. If the effect requires a
/// target, it can be provided via `requested_targets`. Targeted effects with no
/// targets or invalid targets will be ignored.
///
/// # Arguments
///
/// * `battle` - The current battle state.
/// * `source` - The source of the effect.
/// * `effect` - The effect to apply.
/// * `requested_targets` - The targets for the effect.
pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &Effect,
    requested_targets: Option<&EffectTargets>,
) {
    let targets = stack_card_queries::validate_targets(battle, requested_targets);
    match effect {
        Effect::Effect(standard) => {
            apply_standard_effect::apply(battle, source, standard, targets);
            remove_stack_priority_if_empty(battle);
        }
        Effect::WithOptions(with_options) => {
            execute_with_options(battle, source, with_options, targets);
        }
        Effect::List(effects) => {
            assert_that!(
                battle.pending_effects.is_empty(),
                "Pending effects must be empty",
                battle
            );
            battle.pending_effects = effects
                .iter()
                .map(|effect| PendingEffect {
                    source,
                    effect: effect.clone(),
                    targets: targets.cloned(),
                })
                .collect();
            execute_pending_effects_if_no_active_prompt(battle);
        }
    };
}

/// Executes pending effects until there are no more in the queue or a prompt is
/// created.
pub fn execute_pending_effects_if_no_active_prompt(battle: &mut BattleState) {
    loop {
        if !battle.prompts.is_empty() {
            return;
        }

        if matches!(battle.status, BattleStatus::GameOver { .. }) {
            return;
        }

        let Some(pending_effect) = battle.pending_effects.pop_front() else {
            return;
        };

        execute_with_options(
            battle,
            pending_effect.source,
            &pending_effect.effect,
            pending_effect.targets.as_ref(),
        );
    }
}

fn execute_with_options(
    battle: &mut BattleState,
    source: EffectSource,
    with_options: &EffectWithOptions,
    targets: Option<&EffectTargets>,
) {
    if with_options.optional {
        todo!("Implement optional effects")
    }
    if with_options.trigger_cost.is_some() {
        todo!("Implement trigger cost effects")
    }
    if with_options.condition.is_some() {
        todo!("Implement conditional effects")
    }
    apply_standard_effect::apply(battle, source, &with_options.effect, targets);
    remove_stack_priority_if_empty(battle);
}

/// Removes stack priority if the stack is empty.
fn remove_stack_priority_if_empty(battle: &mut BattleState) {
    if !battle.cards.has_stack() {
        battle.stack_priority = None;
    }
}

/// Flattens an Effect into an iterator of PendingEffect.
fn flatten_effect(
    source: EffectSource,
    effect: &Effect,
    targets: Option<EffectTargets>,
) -> impl Iterator<Item = PendingEffect> + '_ {
    match effect {
        Effect::Effect(standard_effect) => Either::Left(iter::once(PendingEffect {
            source,
            effect: EffectWithOptions::new(standard_effect.clone()),
            targets: targets.clone(),
        })),
        Effect::WithOptions(with_options) => Either::Left(iter::once(PendingEffect {
            source,
            effect: with_options.clone(),
            targets: targets.clone(),
        })),
        Effect::List(effects) => Either::Right(effects.iter().map(move |effect| PendingEffect {
            source,
            effect: effect.clone(),
            targets: targets.clone(),
        })),
    }
}
