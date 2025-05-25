use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use core_data::types::PlayerName;
use tracing_macros::panic_with;

/// Returns true if it is the opponent's turn and `player` will win the game
/// in their next judgment phase.
///
/// This functions by simulating the result of the opponent ending their turn
/// and checking if the indicated player has won the game.
pub fn is_victory_imminent_for_player(battle: &BattleState, player: PlayerName) -> bool {
    if battle.turn.active_player == player {
        return false;
    }

    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return false;
    }

    let mut simulation = battle.logical_clone();

    // Clear state which might prevent the 'end turn' action from being legal.
    simulation.prompt = None;
    simulation.stack_priority = None;
    simulation.phase = BattleTurnPhase::Main;

    let opponent = player.opponent();
    let legal_actions = legal_actions::compute(&simulation, opponent);
    if !legal_actions.contains(BattleAction::EndTurn) {
        panic_with!("Opponent cannot end their turn", battle, opponent);
    }

    apply_battle_action::execute(&mut simulation, opponent, BattleAction::EndTurn);

    let legal_actions = legal_actions::compute(&simulation, player);
    if !legal_actions.contains(BattleAction::StartNextTurn) {
        panic_with!("Player cannot start their turn", battle, opponent);
    }
    apply_battle_action::execute(&mut simulation, player, BattleAction::StartNextTurn);

    matches!(simulation.status, BattleStatus::GameOver { winner: Some(winner) } if winner == player)
}
