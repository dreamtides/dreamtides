use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::CharacterId;
use battle_data::battle_cards::zone::Zone;
use core_data::effect_source::EffectSource;

use crate::zone_mutations::move_card;

/// Dissolves a character, moving it to the void.
pub fn apply(battle: &mut BattleData, source: EffectSource, character: CharacterId) {
    move_card::run(battle, source, character, Zone::Void);
}
