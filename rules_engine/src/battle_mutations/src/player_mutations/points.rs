use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use core_data::numerics::Points;
use core_data::source::Source;
use core_data::types::PlayerName;

/// Gains `amount` points for `player`.
pub fn gain(battle: &mut BattleData, player: PlayerName, _source: Source, amount: Points) {
    battle.player_mut(player).points += amount;
    if battle.player(player).points >= Points(6) {
        battle.status = BattleStatus::GameOver { winner: player };
    }
}
