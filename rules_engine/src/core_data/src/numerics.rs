use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A resource used to pay for cards & abilities.
#[derive(
    Debug,
    Display,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Into,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Energy(pub u32);

/// Represents the 'power' of characters; the primary way in which players earn
/// victory points.
#[derive(
    Debug,
    Display,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Into,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Spark(pub u32);

/// Victory points. Enable the player to win the game.
#[derive(
    Debug,
    Display,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Into,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Points(pub u32);

/// Identifies a turn within a game.
#[derive(
    Debug,
    Display,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Sum,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Into,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct TurnId(pub u32);
