use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::{CharacterId, VoidCardId};
use core_data::effect_source::EffectSource;
use tracing::info;

use crate::zone_mutations::move_card;

/// Dissolves a character, moving it to the void.
///
/// Returns the [VoidCardId] for the character if it has been successfully moved
/// to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, id: CharacterId) -> Option<VoidCardId> {
    info!(?id, "Dissolving character");
    move_card::to_void(battle, source, id)
}
