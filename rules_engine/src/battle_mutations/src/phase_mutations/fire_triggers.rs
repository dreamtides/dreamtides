use ability_data::effect::Effect;
use ability_data::predicate::Predicate;
use ability_data::triggered_ability::TriggeredAbility;
use battle_queries::battle_card_queries::{card, card_abilities, card_properties};
use battle_queries::card_ability_queries::{effect_predicates, trigger_queries};
use battle_queries::panic_with;
use battle_state::battle::battle_animation::{BattleAnimation, TriggerAnimation};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::ability_list::AbilityData;
use battle_state::battle_cards::stack_card_state::EffectTargets;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::effects::apply_effect::{self, EffectWasApplied};

/// Fires all recorded triggers for the given [BattleState] while no prompt is
/// active.
///
/// Triggers are removed from the trigger list after firing. Runs until all
/// triggers are resolved, including new triggers that may be added during
/// execution.
pub fn execute_if_no_active_prompt(battle: &mut BattleState) {
    let should_animate = battle.animations.is_some();
    let mut trigger_animations = Vec::new();

    loop {
        if should_animate {
            set_displayed_active_triggers(battle, &mut trigger_animations);
        }

        if battle.prompt.is_some() {
            break;
        }

        let Some(trigger_for_listener) = battle.triggers.events.pop_front() else {
            break;
        };

        let controller = card_properties::controller(battle, trigger_for_listener.listener);
        let Some(character_id) =
            battle.cards.to_character_id(controller, trigger_for_listener.listener)
        else {
            // Skip triggers for cards that are not currently on the
            // battlefield.
            continue;
        };

        for ability_data in &card_abilities::query(battle, character_id).triggered_abilities {
            if trigger_queries::matches(
                battle,
                trigger_for_listener.trigger,
                &ability_data.ability.trigger,
                controller,
                trigger_for_listener.listener,
            ) {
                fire_triggered_ability(
                    battle,
                    trigger_for_listener.trigger,
                    ability_data,
                    controller,
                    character_id,
                );
            }
        }
    }

    if should_animate {
        battle.push_animation(EffectSource::Game { controller: battle.turn.active_player }, || {
            BattleAnimation::SetActiveTriggers { triggers: vec![] }
        });
    }
}

/// Sets the active trigger display state for all valid triggers in
/// [BattleState] and adds them to `previous`.
///
/// The animation is skipped if the set of active triggers is the same as
/// `previous`.
#[cold]
fn set_displayed_active_triggers(battle: &mut BattleState, previous: &mut Vec<TriggerAnimation>) {
    let mut trigger_animations = Vec::new();
    for trigger_for_listener in &battle.triggers.events {
        let controller = card_properties::controller(battle, trigger_for_listener.listener);
        let Some(character_id) =
            battle.cards.to_character_id(controller, trigger_for_listener.listener)
        else {
            continue;
        };
        for ability_data in &card_abilities::query(battle, character_id).triggered_abilities {
            if trigger_queries::matches(
                battle,
                trigger_for_listener.trigger,
                &ability_data.ability.trigger,
                controller,
                trigger_for_listener.listener,
            ) {
                trigger_animations.push(TriggerAnimation {
                    controller,
                    character_id,
                    ability_number: ability_data.ability_number,
                });
            }
        }
    }

    if trigger_animations != *previous {
        battle.push_animation(EffectSource::Game { controller: battle.turn.active_player }, || {
            BattleAnimation::SetActiveTriggers { triggers: trigger_animations.clone() }
        });
    }

    *previous = trigger_animations;
}

/// Fires a triggered ability for the given [BattleState].
///
/// Returns true if an effect
fn fire_triggered_ability(
    battle: &mut BattleState,
    trigger: Trigger,
    ability_data: &AbilityData<TriggeredAbility>,
    controller: PlayerName,
    controlling_character: CharacterId,
) -> Option<EffectWasApplied> {
    let source = EffectSource::Triggered {
        controller,
        character_id: controlling_character,
        ability_number: ability_data.ability_number,
    };
    let Effect::Effect(effect) = &ability_data.ability.effect else {
        todo!("Implement non-standard triggered effects")
    };

    let targets = trigger_targets(battle, trigger, effect, controller, controlling_character);
    apply_effect::execute(battle, source, &ability_data.ability.effect, targets.as_ref())
}

fn trigger_targets(
    battle: &BattleState,
    trigger: Trigger,
    effect: &ability_data::standard_effect::StandardEffect,
    controller: PlayerName,
    controlling_character: CharacterId,
) -> Option<EffectTargets> {
    let mut targets = None;

    if let Some(predicate) = effect_predicates::get_character_target_predicate(effect) {
        match predicate {
            Predicate::This => {
                targets = Some(EffectTargets::Character(
                    controlling_character,
                    card::get(battle, controlling_character).object_id,
                ));
            }
            Predicate::That => {
                let Some(triggering_card_id) = trigger_queries::triggering_card_id(trigger) else {
                    panic_with!("Expected a triggering card ID", battle);
                };
                let Some(target_character_id) =
                    battle.cards.to_character_id(controller, triggering_card_id)
                else {
                    // Skip triggers targeting cards that are not currently on
                    // the battlefield.
                    return None;
                };
                targets = Some(EffectTargets::Character(
                    target_character_id,
                    card::get(battle, target_character_id).object_id,
                ));
            }
            _ => todo!("Implement trigger target selection for {:?}", predicate),
        }
    }

    if let Some(predicate) = effect_predicates::get_stack_target_predicate(effect) {
        match predicate {
            Predicate::That => {
                let Some(triggering_card_id) = trigger_queries::triggering_card_id(trigger) else {
                    panic_with!("Expected a triggering card ID", battle);
                };
                let Some(target_stack_card_id) =
                    battle.cards.to_stack_card_id(controller, triggering_card_id)
                else {
                    // Skip triggers targeting cards that are not currently on the stack.
                    return None;
                };
                targets = Some(EffectTargets::StackCard(
                    target_stack_card_id,
                    card::get(battle, target_stack_card_id).object_id,
                ));
            }
            _ => todo!("Implement trigger stack target selection for {:?}", predicate),
        }
    }

    targets
}
