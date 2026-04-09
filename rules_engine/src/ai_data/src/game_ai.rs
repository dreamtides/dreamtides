use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    MonteCarlo(u32),
    MonteCarloSingleThreaded(u32),
    MonteCarloV2(u32),
    MonteCarloV3(u32),
    MonteCarloV4(u32),
    MonteCarloHybridV1(u32),
    StrategicV1(u32),
    StrategicV2(u32),
    StrategicV3(u32),
    WaitFiveSeconds,
}

// I've tested more traditional tree search algorithms like Minimax/Alpha
// Beta pruning, but they've been very convincingly defeated by Monte Carlo
// techniques.
