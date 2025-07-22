use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_trace;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;

use crate::card_mutations::move_card;

/// Attempts to dissolves a character, moving it to the void.
///
/// Panics if the character is not on the battlefield.
pub fn execute(battle: &mut BattleState, source: EffectSource, id: CharacterId) -> VoidCardId {
    battle_trace!("Dissolving character", battle, id);
    battle.push_animation(source, || BattleAnimation::Dissolve { target_id: id });
    let id = move_card::from_battlefield_to_void(
        battle,
        source,
        card_properties::controller(battle, id),
        id,
    );
    battle.triggers.push(source, Trigger::Dissolved(id));
    id
}
