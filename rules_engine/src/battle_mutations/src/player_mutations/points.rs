use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Points;
use core_data::types::PlayerName;

/// Gains `amount` points for `player`.
pub fn gain(battle: &mut BattleState, player: PlayerName, source: EffectSource, amount: Points) {
    battle.push_animation(source, || BattleAnimation::ScorePoints { player, source });
    let player_state = battle.players.player_mut(player);
    player_state.points += amount;
    if player_state.points >= battle.rules_config.points_to_win {
        battle.status = BattleStatus::GameOver { winner: Some(player) };
    }
}
