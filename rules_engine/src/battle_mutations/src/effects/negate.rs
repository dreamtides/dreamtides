use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::{StackCardId, VoidCardId};
use core_data::effect_source::EffectSource;
use tracing::info;

use crate::zone_mutations::move_card;

/// Negates a card on the stack, moving it to the void.
///
/// Returns the [VoidCardId] for the character if it has been successfully moved
/// to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, id: StackCardId) -> Option<VoidCardId> {
    info!(?id, "Negating card");
    move_card::to_void(battle, source, id)
}
