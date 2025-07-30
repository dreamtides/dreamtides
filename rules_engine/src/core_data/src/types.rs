use enumset::EnumSetType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Whether a card is face-down or face-up
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum CardFacing {
    FaceDown,
    FaceUp,
}

/// Identifies a player in an ongoing battle.
#[derive(Debug, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema, EnumSetType)]
pub enum PlayerName {
    One,
    Two,
}

impl PlayerName {
    pub fn opponent(self) -> PlayerName {
        match self {
            PlayerName::One => PlayerName::Two,
            PlayerName::Two => PlayerName::One,
        }
    }
}
