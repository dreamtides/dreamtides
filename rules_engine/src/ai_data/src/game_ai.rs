use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    IterativeDeepening,
    Uct1,
    Uct1MaxIterations(u32),
    NewUct(u32),
}
