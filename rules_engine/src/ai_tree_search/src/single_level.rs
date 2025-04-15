use ai_core::agent::AgentConfig;
use ai_core::game_state_node::GameStateNode;
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;

/// An agent which does a depth 1 search of legal actions, returning the one
/// that produces the best evaluator state.
pub struct SingleLevel {}

impl SelectionAlgorithm for SingleLevel {
    fn pick_action<N, E>(
        &self,
        _: AgentConfig,
        node: &N,
        evaluator: &E,
        player: N::PlayerName,
    ) -> N::Action
    where
        N: GameStateNode,
        E: StateEvaluator<N>,
    {
        let mut best_score = i32::MIN;
        let mut best_action: Option<N::Action> = None;

        for action in node.legal_actions(player) {
            let mut child = node.make_copy();
            child.execute_action(player, action);
            let score = evaluator.evaluate(&child, player);
            if score > best_score {
                best_score = score;
                best_action = Some(action);
            }
        }

        best_action.expect("No legal actions found")
    }
}
