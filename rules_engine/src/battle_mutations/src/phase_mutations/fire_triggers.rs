use ability_data::triggered_ability::TriggeredAbility;
use battle_queries::battle_card_queries::{card, card_properties};
use battle_queries::card_ability_queries::trigger_queries;
use battle_state::battle::battle_animation_data::{BattleAnimation, TriggerAnimation};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::ability_list::AbilityData;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::effects::apply_effect_with_prompt_for_targets;

/// Fires all recorded triggers for the given [BattleState] while no prompt is
/// active.
///
/// Triggers are removed from the trigger list after firing. Runs until all
/// triggers are resolved, including new triggers that may be added during
/// execution.
#[inline]
pub fn execute_if_no_active_prompt(battle: &mut BattleState) {
    if !battle.triggers.events.is_empty() && battle.prompts.is_empty() {
        execute_if_no_active_prompt_internal(battle);
    }
}

#[cold]
fn execute_if_no_active_prompt_internal(battle: &mut BattleState) {
    let should_animate = battle.animations.is_some();
    let mut trigger_animations = Vec::new();

    loop {
        if !battle.prompts.is_empty() {
            break;
        }

        if matches!(battle.status, BattleStatus::GameOver { .. }) {
            break;
        }

        if should_animate {
            set_displayed_active_triggers(battle, &mut trigger_animations);
        }

        let Some(trigger_for_listener) = battle.triggers.events.pop_front() else {
            break;
        };

        let controller = card_properties::controller(battle, trigger_for_listener.listener);

        // Check whether this character is currently on the battlefield.
        let Some(character_id) =
            battle.cards.to_character_id(controller, trigger_for_listener.listener)
        else {
            continue;
        };

        for ability_data in &card::ability_list(battle, character_id).triggered_abilities {
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

    if should_animate && battle.prompts.is_empty() {
        // TODO: Handle updating active triggers when resolving a prompt pending
        // effect
        set_displayed_active_triggers(battle, &mut trigger_animations);
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
        for ability_data in &card::ability_list(battle, character_id).triggered_abilities {
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
) {
    let source = EffectSource::Triggered {
        controller,
        character_id: controlling_character,
        ability_number: ability_data.ability_number,
    };

    let that_card = trigger_queries::triggering_card_id(trigger);
    apply_effect_with_prompt_for_targets::execute(
        battle,
        source,
        &ability_data.ability.effect,
        that_card,
        None,
    );
}
