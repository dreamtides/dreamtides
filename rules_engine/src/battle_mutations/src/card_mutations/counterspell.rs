use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_trace;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::StackCardId;
use battle_state::core::effect_source::EffectSource;

use crate::card_mutations::move_card;

/// Prevents a card on the stack from resolving, moving it to the void.
///
/// Panics if the card is not on the stack.
pub fn execute(battle: &mut BattleState, source: EffectSource, id: StackCardId) {
    battle_trace!("Counterspelling card", battle, id);
    battle.push_animation(source, || BattleAnimation::Counterspell { target_id: id });
    move_card::from_stack_to_void(battle, source, card_properties::controller(battle, id), id);
}
