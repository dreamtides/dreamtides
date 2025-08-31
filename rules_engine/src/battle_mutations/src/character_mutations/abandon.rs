use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_trace;
use battle_state::battle::battle_animation_data::{BattleAnimation, TargetedEffectName};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;

use crate::card_mutations::move_card;

/// Moves a character to the void. Cannot be prevented.
///
/// Panics if the character is not on the battlefield.
pub fn apply(battle: &mut BattleState, source: EffectSource, id: CharacterId) -> VoidCardId {
    battle_trace!("Abandoning character", battle, id);
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
    id
}
