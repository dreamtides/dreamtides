use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    MonteCarlo(u32),
    MonteCarloSingleThreaded(u32),
    // I've tested more traditional tree search algorithms like Minimax/Alpha
    // Beta pruning, but they've been very convincingly defeated by Monte Carlo
    // techniques.
}
