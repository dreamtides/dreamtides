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

/// Identifies a player in an ongoing battle.
#[derive(
    Clone, Debug, Eq, PartialEq, Copy, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum PlayerName {
    /// Player who is currently operating the client
    User,

    /// Opponent of user, i.e. the AI enemy
    Enemy,
}
