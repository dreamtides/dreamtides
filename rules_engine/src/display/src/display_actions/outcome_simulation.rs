use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use core_data::types::PlayerName;
use display_data::battle_view::{BattlePreviewView, PlayerPreviewView};
use tracing_macros::panic_with;
use ui_components::component::Component;
use ui_components::icon;

use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};

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

/// Returns a preview of the battle state based on simulating the effect of
/// playing the given card.
///
/// Returns None if no changes are detected between the simulated state and the
/// current state.
pub fn action_effect_preview(
    battle: &BattleState,
    player: PlayerName,
    action: BattleAction,
) -> Option<BattlePreviewView> {
    let mut simulation = battle.logical_clone();
    apply_battle_action::execute(&mut simulation, player, action);
    if simulation.turn_history.current_action_history.player(player).hand_size_limit_exceeded {
        return Some(BattlePreviewView {
            user: PlayerPreviewView::default(),
            enemy: PlayerPreviewView::default(),
            cards: vec![],
            preview_message: hand_size_limit_exceeded_message().flex_node(),
        });
    }

    None
}

fn hand_size_limit_exceeded_message() -> impl Component {
    InterfaceMessage::builder()
        .text(format!("Note: cards drawn in excess of 10 become {} instead.", icon::ENERGY))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}
