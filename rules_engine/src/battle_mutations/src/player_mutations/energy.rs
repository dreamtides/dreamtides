use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use tracing_macros::assert_that;

pub fn spend(battle: &mut BattleState, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.players.player_mut(player);
    assert_that!(
        player_state.current_energy >= amount,
        "Player has insufficient energy",
        battle,
        player,
        amount
    );
    player_state.current_energy -= amount;
}

/// Sets `player`'s current energy to `amount`.
pub fn set(battle: &mut BattleState, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.players.player_mut(player);
    player_state.current_energy = amount;
}

/// Sets `player`'s produced energy to `amount`.
pub fn set_produced(
    battle: &mut BattleState,
    player: PlayerName,
    _source: EffectSource,
    amount: Energy,
) {
    let player_state = battle.players.player_mut(player);
    player_state.produced_energy = amount;
}

/// Adds `amount` to `player`'s current energy.
pub fn gain(battle: &mut BattleState, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.players.player_mut(player);
    player_state.current_energy += amount;
}
