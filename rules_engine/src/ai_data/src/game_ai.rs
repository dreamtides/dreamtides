use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    IterativeDeepening,
    Uct1,
    Uct1MaxIterations(u32),
}
