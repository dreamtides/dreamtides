use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies a human player of the game.
///
/// Equivalently, this identifies a save file.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct UserId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct BattleId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct QuestId(pub Uuid);

/// Identifies the base rules for a card.
///
/// The 'base card' describes the abilities of a card before any modifications
/// are applied to it, i.e. a card that appears in the Tabula database.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct BaseCardId(pub Uuid);

/// Identifies a dreamwell card.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct DreamwellCardId(pub Uuid);

/// Number of an ability within a card.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct AbilityNumber(pub usize);

/// Identifies a location with an effect within a dreamscape.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct SiteId(pub Uuid);
