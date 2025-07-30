use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    MonteCarlo(u32),
    MonteCarloSingleThreaded(u32),
    WaitFiveSeconds,
}

// I've tested more traditional tree search algorithms like Minimax/Alpha
// Beta pruning, but they've been very convincingly defeated by Monte Carlo
// techniques.
