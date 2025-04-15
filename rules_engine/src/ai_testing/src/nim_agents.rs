use ai_core::agent::AgentData;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use ai_tree_search::single_level::SingleLevel;

use crate::nim::{NimPerfectEvaluator, NimState, NimWinLossEvaluator};

/// Agent which always makes optimal moves
pub const NIM_PERFECT_AGENT: AgentData<SingleLevel, NimPerfectEvaluator, NimState> =
    AgentData::omniscient("PERFECT", SingleLevel {}, NimPerfectEvaluator {});

pub const NIM_MINIMAX_AGENT: AgentData<MinimaxAlgorithm, NimWinLossEvaluator, NimState> =
    AgentData::omniscient("MINIMAX", MinimaxAlgorithm { search_depth: 25 }, NimWinLossEvaluator {});

pub const NIM_ALPHA_BETA_AGENT: AgentData<AlphaBetaAlgorithm, NimWinLossEvaluator, NimState> =
    AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

pub const NIM_UCT1_AGENT: AgentData<MonteCarloAlgorithm<Uct1>, RandomPlayoutEvaluator, NimState> =
    AgentData::omniscient(
        "UCT1",
        MonteCarloAlgorithm { child_score_algorithm: Uct1 {} },
        RandomPlayoutEvaluator {},
    );
