use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{StackCardId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::card_mutations::move_card;

/// Negates a card on the stack, moving it to the void.
///
/// Returns the [VoidCardId] for the card in its new zone.
pub fn execute(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    id: StackCardId,
) -> VoidCardId {
    battle_trace!("Negating card", battle, id);
    move_card::from_stack_to_void(battle, source, controller, id)
}
