use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

/// A User ID
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct UserId(pub Uuid);

/// A Battle ID
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct BattleId(pub Uuid);

/// A URL
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Url(pub String);

/// Whether a card is face-down or face-up
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CardFacing {
    FaceDown,
    FaceUp,
}
