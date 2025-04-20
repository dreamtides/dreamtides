use ability_data::cost::Cost;
use battle_data::battle::battle_data::BattleData;
use core_data::types::PlayerName;

/// Returns true if the [PlayerName] player can pay a [Cost].
pub fn can_pay(battle: &BattleData, player: PlayerName, cost: &Cost) -> bool {
    match cost {
        Cost::Energy(energy) => battle.player(player).current_energy >= *energy,
        _ => todo!("Implement {:?}", cost),
    }
}
