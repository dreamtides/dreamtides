use ability_data::ability::EventAbility;
use ability_data::effect::{Effect, EffectWithOptions, ModalEffectChoice, ModelEffectChoiceIndex};
use battle_queries::battle_card_queries::target_queries;
use battle_queries::panic_with;
use battle_state::battle::battle_state::{BattleState, PendingEffect};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::card_id::StackCardId;
use battle_state::battle_cards::ability_list::AbilityData;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

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
    modal_choice: Option<ModelEffectChoiceIndex>,
) {
    match abilities {
        [] => {}
        [ability] => {
            let source = EffectSource::Event {
                controller,
                stack_card_id,
                ability_number: ability.ability_number,
            };
            execute(battle, source, &ability.ability.effect, requested_targets, modal_choice);
        }
        _ => {
            battle.pending_effects.extend(abilities.iter().map(|ability_data| {
                let source = EffectSource::Event {
                    controller,
                    stack_card_id,
                    ability_number: ability_data.ability_number,
                };
                PendingEffect {
                    source,
                    effect: ability_data.ability.effect.clone(),
                    requested_targets: requested_targets.cloned(),
                    modal_choice,
                }
            }));
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
    modal_choice: Option<ModelEffectChoiceIndex>,
) {
    match effect {
        Effect::Effect(standard) => {
            let mut targets = target_queries::valid_targets(battle, requested_targets);
            apply_standard_effect::apply(battle, source, standard, &mut targets);
            remove_stack_priority_if_empty(battle);
        }
        Effect::WithOptions(with_options) => {
            let mut targets = target_queries::valid_targets(battle, requested_targets);
            execute_with_options(battle, source, with_options, &mut targets);
        }
        Effect::List(_) => {
            battle.pending_effects.push_back(PendingEffect {
                source,
                effect: effect.clone(),
                requested_targets: requested_targets.cloned(),
                modal_choice,
            });
            execute_pending_effects_if_no_active_prompt(battle);
        }
        Effect::Modal(choices) => {
            if let Some(modal_choice) = modal_choice {
                execute_modal_effect(battle, source, choices, requested_targets, modal_choice);
            } else {
                panic_with!("Modal effect requires an effect choice", battle);
            }
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

        match pending_effect.effect {
            Effect::Effect(standard) => {
                let mut targets = target_queries::valid_targets(
                    battle,
                    pending_effect.requested_targets.as_ref(),
                );
                apply_standard_effect::apply(
                    battle,
                    pending_effect.source,
                    &standard,
                    &mut targets,
                );
                remove_stack_priority_if_empty(battle);
            }
            Effect::WithOptions(with_options) => {
                let mut targets = target_queries::valid_targets(
                    battle,
                    pending_effect.requested_targets.as_ref(),
                );
                execute_with_options(battle, pending_effect.source, &with_options, &mut targets);
            }
            Effect::List(mut effect_list) => {
                if !effect_list.is_empty() {
                    let first_effect = effect_list.remove(0);
                    let mut targets = target_queries::valid_targets(
                        battle,
                        pending_effect.requested_targets.as_ref(),
                    );
                    execute_with_options(
                        battle,
                        pending_effect.source,
                        &first_effect,
                        &mut targets,
                    );

                    if !effect_list.is_empty() {
                        battle.pending_effects.push_front(PendingEffect {
                            source: pending_effect.source,
                            effect: Effect::List(effect_list),
                            requested_targets: pending_effect.requested_targets,
                            modal_choice: pending_effect.modal_choice,
                        });
                    }
                }
            }
            Effect::Modal(choices) => {
                if let Some(modal_choice) = pending_effect.modal_choice {
                    execute_modal_effect(
                        battle,
                        pending_effect.source,
                        &choices,
                        pending_effect.requested_targets.as_ref(),
                        modal_choice,
                    );
                } else {
                    panic_with!("Modal effect requires an effect choice", battle);
                }
            }
        }
    }
}

fn execute_modal_effect(
    _battle: &mut BattleState,
    _source: EffectSource,
    _effects: &[ModalEffectChoice],
    _requested_targets: Option<&EffectTargets>,
    _modal_choice: ModelEffectChoiceIndex,
) -> Option<EffectWasApplied> {
    todo!("Implement modal effects")
}

fn execute_with_options(
    battle: &mut BattleState,
    source: EffectSource,
    with_options: &EffectWithOptions,
    targets: &mut Option<EffectTargets>,
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
