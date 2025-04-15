use crate::game_state_node::{GameStateNode, GameStatus};
use crate::state_evaluator::StateEvaluator;
use crate::state_predictor::StatePredictor;

/// StateCombiners merge together the results of [StatePredictor]s. Given the
/// set of predicted game states, they select the best *actual* game state to
/// use as the basis for a selection algorithm.
pub type StateCombiner<TNode, TEvaluator> = fn(&TNode, StatePredictor<TNode>, &TEvaluator) -> TNode;

/// A pessimistic [StateCombiner]. Evaluates each predicted game state and picks
/// the one which appears to be the worst-case scenario (i.e. has the lowest
/// [StateEvaluator] score) for the current player.
pub fn worst_case<TEvaluator, TNode>(
    node: &TNode,
    predictor: StatePredictor<TNode>,
    evaluator: &TEvaluator,
) -> TNode
where
    TEvaluator: StateEvaluator<TNode>,
    TNode: GameStateNode,
{
    let player = match node.status() {
        GameStatus::InProgress { current_turn } => current_turn,
        _ => panic!("Game is over"),
    };
    let mut worst = i32::MAX;
    let mut worst_state: Option<TNode> = None;
    for state in predictor(node) {
        let evaluation = evaluator.evaluate(&state, player);
        if evaluation < worst {
            worst = evaluation;
            worst_state = Some(state);
        }
    }

    worst_state.expect("No state found")
}
