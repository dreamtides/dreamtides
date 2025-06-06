use battle_queries::battle_card_queries::card_properties;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use tracing_macros::battle_trace;

use crate::card_mutations::move_card;

/// Dissolves a character, moving it to the void.
pub fn execute(battle: &mut BattleState, source: EffectSource, id: CharacterId) {
    battle_trace!("Dissolving character", battle, id);
    battle.push_animation(source, || BattleAnimation::Dissolve { target_id: id });
    move_card::from_battlefield_to_void(
        battle,
        source,
        card_properties::controller(battle, id),
        id,
    );
}
