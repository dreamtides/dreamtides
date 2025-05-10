use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::character_mutations::dissolve;

/// Dissolves a character, moving it to the void.
pub fn apply(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    id: CharacterId,
) {
    battle_trace!("Abandoning character", battle, id);
    dissolve::execute(battle, source, controller, id);
}
