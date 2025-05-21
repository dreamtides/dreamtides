use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameAI {
    AlwaysPanic,
    FirstAvailableAction,
    RandomAction,
    IterativeDeepening,
    OldUct1,
    OldUct1MaxIterations(u32),
    Uct1(u32),
    Uct1SingleThreaded(u32),
}
