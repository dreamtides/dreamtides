use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_queries::card_ability_queries::trigger_queries;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;

use crate::effects::apply_effect;

/// Fires all recorded triggers for the given [BattleState] while no prompt is
/// active.
///
/// Triggers are removed from the trigger list after firing. Runs until all
/// triggers are resolved, including new triggers that may be added during
/// execution.
pub fn execute_if_no_active_prompt(battle: &mut BattleState) {
    loop {
        if battle.prompt.is_some() {
            break;
        }

        let Some(trigger) = battle.triggers.events.pop_front() else {
            break;
        };

        let controller = card_properties::controller(battle, trigger.listener);
        let Some(character_id) = battle.cards.to_character_id(controller, trigger.listener) else {
            // Skip triggers for cards that are not currently on the
            // battlefield.
            continue;
        };

        for ability_data in &card_abilities::query(battle, character_id).triggered_abilities {
            if trigger_queries::matches(
                battle,
                trigger.trigger,
                &ability_data.ability.trigger,
                controller,
                trigger.listener,
            ) {
                let source = EffectSource::Triggered {
                    controller,
                    character_id,
                    ability_number: ability_data.ability_number,
                };
                apply_effect::execute(battle, source, &ability_data.ability.effect, None);
            }
        }
    }
}
