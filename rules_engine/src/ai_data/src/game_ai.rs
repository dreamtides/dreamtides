use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAI {
    FirstAvailableAction,
    RandomAction,
    IterativeDeepening,
    Uct1,
}
