use crate::game_state_node::{GameStateNode, GameStatus};
use crate::state_evaluator::StateEvaluator;

/// A StateEvaluator which combines together the results of a list of child
/// evaluators, multiplying each result by the associated weight.
///
/// Automatically handles the 'game over' state by returning i32::MAX/i32::MIN
/// if the player won/lost the game.
pub struct CompoundEvaluator<TNode: GameStateNode> {
    pub evaluators: Vec<(i32, Box<dyn StateEvaluator<TNode>>)>,
}

impl<TNode: GameStateNode> StateEvaluator<TNode> for CompoundEvaluator<TNode> {
    fn evaluate(&self, node: &TNode, player: TNode::PlayerName) -> i32 {
        if let GameStatus::Completed { winner } = node.status() {
            return if winner == player { i32::MAX } else { i32::MIN };
        }

        let mut score = 0;
        for (weight, evaluator) in &self.evaluators {
            score += weight * evaluator.evaluate(node, player);
        }
        score
    }
}
