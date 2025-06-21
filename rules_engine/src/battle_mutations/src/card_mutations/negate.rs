use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_trace;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{StackCardId, VoidCardId};
use battle_state::core::effect_source::EffectSource;

use crate::card_mutations::move_card;

/// Negates a card on the stack, moving it to the void.
///
/// Returns the [VoidCardId] for the card in its new zone.
pub fn execute(battle: &mut BattleState, source: EffectSource, id: StackCardId) -> VoidCardId {
    battle_trace!("Negating card", battle, id);
    battle.push_animation(source, || BattleAnimation::Negate { target_id: id });
    move_card::from_stack_to_void(battle, source, card_properties::controller(battle, id), id)
}
