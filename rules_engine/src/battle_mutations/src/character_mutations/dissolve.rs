use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::{CharacterId, VoidCardId};
use logging::battle_trace;

use crate::zone_mutations::move_card;

/// Dissolves a character, moving it to the void.
///
/// Returns the [VoidCardId] for the character if it has been successfully moved
/// to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, id: CharacterId) -> Option<VoidCardId> {
    battle_trace!("Dissolving character", battle, id);
    move_card::to_void(battle, source, id)
}
