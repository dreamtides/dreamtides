use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_trace;
use battle_queries::card_ability_queries::effect_queries;
use battle_state::battle::battle_animation::{BattleAnimation, TargetedEffectName};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;

use crate::card_mutations::move_card;

/// Attempts to dissolve a character, moving it to the void. Returns `None` if
/// the dissolve is prevented.
///
/// Panics if the character is not on the battlefield.
pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    id: CharacterId,
) -> Option<VoidCardId> {
    if effect_queries::should_prevent_dissolve(battle, id) {
        battle_trace!("Prevented dissolve", battle, id);
        battle
            .push_animation(source, || BattleAnimation::PreventedEffect { card_id: id.card_id() });
        None
    } else {
        battle_trace!("Dissolving character", battle, id);
        battle.push_animation(source, || BattleAnimation::ApplyTargetedEffect {
            effect_name: TargetedEffectName::Dissolve,
            targets: vec![id.card_id()],
        });
        let id = move_card::from_battlefield_to_void(
            battle,
            source,
            card_properties::controller(battle, id),
            id,
        );
        battle.triggers.push(source, Trigger::Dissolved(id));
        Some(id)
    }
}
