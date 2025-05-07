use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle_cards::card_id::{CharacterId, VoidCardId};

use crate::character_mutations::dissolve;

/// To 'abandon' a character is to dissolve a character you control, moving it
/// to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, id: CharacterId) -> Option<VoidCardId> {
    dissolve::apply(battle, source, id)
}
