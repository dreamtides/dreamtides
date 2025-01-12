use serde::{Deserialize, Serialize};
use specta::Type;

/// A URL
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Url(pub String);

/// Whether a card is face-down or face-up
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CardFacing {
    FaceDown,
    FaceUp,
}
