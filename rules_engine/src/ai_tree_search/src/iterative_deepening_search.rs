use std::time::Instant;

use ai_core::agent::AgentConfig;
use ai_core::game_state_node::GameStateNode;
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;

use crate::alpha_beta;

/// Implements a search algorithm which repeatedly applies alpha-beta search at
/// increasing depths until its deadline is exceeded
#[derive(Debug, Clone)]
pub struct IterativeDeepeningSearch;

impl SelectionAlgorithm for IterativeDeepeningSearch {
    fn pick_action<N, E>(
        &self,
        config: AgentConfig,
        node: &N,
        evaluator: &E,
        player: N::PlayerName,
    ) -> N::Action
    where
        N: GameStateNode,
        E: StateEvaluator<N>,
    {
        let mut depth = 1;
        let mut best_action = None;

        while config.deadline > Instant::now() {
            let result = alpha_beta::run_internal(
                config,
                node,
                evaluator,
                depth,
                player,
                i32::MIN,
                i32::MAX,
            );
            best_action = Some(result.action());
            depth += 1;
        }

        best_action.expect("No action found")
    }
}
