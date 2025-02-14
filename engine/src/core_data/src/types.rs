use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Whether a card is face-down or face-up
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CardFacing {
    FaceDown,
    FaceUp,
}
