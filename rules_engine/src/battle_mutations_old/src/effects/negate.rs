use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_cards::card_id::{StackCardId, VoidCardId};
use logging::battle_trace;

use crate::zone_mutations::move_card;

/// Negates a card on the stack, moving it to the void.
///
/// Returns the [VoidCardId] for the character if it has been successfully moved
/// to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, id: StackCardId) -> Option<VoidCardId> {
    battle_trace!("Negating card", battle, id);
    move_card::to_void(battle, source, id)
}
