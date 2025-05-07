use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::card_mutations::move_card;

/// Dissolves a character, moving it to the void.
///
/// Returns the [VoidCardId] for the character if it has been successfully moved
/// to the void.
pub fn apply(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    id: CharacterId,
) {
    battle_trace!("Dissolving character", battle, id);
    move_card::from_battlefield_to_void(battle, source, controller, id);
}
