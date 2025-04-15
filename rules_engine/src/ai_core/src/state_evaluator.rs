use crate::game_state_node::GameStateNode;

/// A trait implementation for producing a 'score' for a given game state.
pub trait StateEvaluator<TNode: GameStateNode>: Send {
    /// Evaluate the heuristic value of the given game state for the provided
    /// `player`, returning a higher number for "better" game states.
    ///
    /// Evaluators are expected to return valid results for all game states,
    /// regardless of whether the game has ended, but can return 0 as a default
    /// response.
    ///
    /// For example, the simplest evaluator might return 0 for all in-progress
    /// game states and 1 or -1 based on whether the indicated player won
    /// the game.
    fn evaluate(&self, node: &TNode, player: TNode::PlayerName) -> i32;
}

/// An evaluator which always returns 0.
pub struct ZeroEvaluator {}

impl<TNode: GameStateNode> StateEvaluator<TNode> for ZeroEvaluator {
    fn evaluate(&self, _: &TNode, _: TNode::PlayerName) -> i32 {
        0
    }
}
