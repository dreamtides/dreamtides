use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

pub fn spend(battle: &mut BattleState, player: PlayerName, _source: EffectSource, amount: Energy) {
    let player_state = battle.players.player_mut(player);
    // assert_that!(player_state.current_energy >= amount, battle, || format!(
    //     "{:?} has insufficient energy to pay {:?}",
    //     player, amount
    // ));
    player_state.current_energy -= amount;
}
