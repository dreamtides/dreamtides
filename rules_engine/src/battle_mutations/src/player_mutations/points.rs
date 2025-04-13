use battle_data::battle::battle_data::BattleData;
use core_data::numerics::Points;
use core_data::source::Source;
use core_data::types::PlayerName;

/// Gains `amount` points for `player`.
pub fn gain(battle: &mut BattleData, player: PlayerName, _source: Source, amount: Points) {
    battle.player_mut(player).points += amount;
}
