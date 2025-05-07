use assert_with::assert_that;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

/// Spends `amount` energy from `player`'s current energy.
///
/// Panics if `player` has insufficient energy available.
pub fn spend(battle: &mut BattleData, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.player_mut(player);
    assert_that!(player_state.current_energy >= amount, battle, || format!(
        "{:?} has insufficient energy to pay {:?}",
        player, amount
    ));
    player_state.current_energy -= amount;
}

/// Sets `player`'s current energy to `amount`.
pub fn set(battle: &mut BattleData, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.player_mut(player);
    player_state.current_energy = amount;
}

/// Sets `player`'s produced energy to `amount`.
pub fn set_produced(
    battle: &mut BattleData,
    player: PlayerName,
    _source: EffectSource,
    amount: Energy,
) {
    let player_state = battle.player_mut(player);
    player_state.produced_energy = amount;
}

/// Adds `amount` to `player`'s current energy.
pub fn gain(battle: &mut BattleData, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.player_mut(player);
    player_state.current_energy += amount;
}
