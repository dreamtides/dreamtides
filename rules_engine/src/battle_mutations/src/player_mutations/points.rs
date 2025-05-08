use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Points;
use core_data::types::PlayerName;

/// Gains `amount` points for `player`.
pub fn gain(battle: &mut BattleState, player: PlayerName, _source: EffectSource, amount: Points) {
    let player_state = battle.players.player_mut(player);
    player_state.points += amount;
    if player_state.points >= Points(25) {
        battle.status = BattleStatus::GameOver { winner: player };
    }
}
