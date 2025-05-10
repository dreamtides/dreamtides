use ability_data::cost::Cost;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;

/// Returns true if the [PlayerName] player can pay a [Cost].
pub fn can_pay(battle: &BattleState, player: PlayerName, cost: &Cost) -> bool {
    match cost {
        Cost::Energy(energy) => battle.players.player(player).current_energy >= *energy,
        _ => todo!("Implement {:?}", cost),
    }
}
