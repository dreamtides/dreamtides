use battle_data::battle::battle_data::BattleData;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

/// Spends `amount` energy from `player`'s current energy.
///
/// Panics if `player` has insufficient energy available.
pub fn spend(battle: &mut BattleData, player: PlayerName, amount: Energy) {
    let player_state = battle.player_mut(player);
    assert!(player_state.current_energy >= amount);
    player_state.current_energy -= amount;
}
