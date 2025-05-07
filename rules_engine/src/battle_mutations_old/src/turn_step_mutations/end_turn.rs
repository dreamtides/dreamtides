use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::battle_turn_step::BattleTurnStep;
use logging::battle_trace;

/// End the current player's turn.
///
/// Their opponent may take 'fast' actions before beginning a new turn.
pub fn run(battle: &mut BattleData) {
    battle.step = BattleTurnStep::Ending;
    battle_trace!("Moving to end step for player", battle, player = battle.turn.active_player);
    battle.priority = battle.turn.active_player.opponent();
}
