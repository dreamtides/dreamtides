use crate::agent::AgentConfig;
use crate::game_state_node::GameStateNode;
use crate::state_evaluator::StateEvaluator;

/// A trait for generic decision rules that select a game action to take without
/// specific game knowledge.
pub trait SelectionAlgorithm: Send {
    /// Should return the best action action for the current player `player`
    /// to take in the provided `node` game state, using the provided
    /// `evaluator` to evaluate different game outcomes.
    ///
    /// Implementations are expected to return a result before the
    /// `config.deadline` time by periodically comparing it to
    /// `Instant::now()`.
    fn pick_action<TStateNode, TEvaluator>(
        &self,
        config: AgentConfig,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> TStateNode::Action
    where
        TStateNode: GameStateNode,
        TEvaluator: StateEvaluator<TStateNode>;
}
