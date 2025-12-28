use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};
use fluent::FluentValue;
use fluent::types::FluentNumber;
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
pub struct Points(pub u32);

/// Currency used during quests.
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
pub struct Essence(pub u32);

/// Identifies a turn within a game.
///
/// Turn 0 is the starting player's first turn.
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
pub struct TurnId(pub u32);

impl<'a> From<Energy> for FluentValue<'a> {
    fn from(energy: Energy) -> Self {
        FluentValue::Number(FluentNumber::from(energy.0))
    }
}

impl<'a> From<Spark> for FluentValue<'a> {
    fn from(spark: Spark) -> Self {
        FluentValue::Number(FluentNumber::from(spark.0))
    }
}

impl<'a> From<Points> for FluentValue<'a> {
    fn from(points: Points) -> Self {
        FluentValue::Number(FluentNumber::from(points.0))
    }
}

impl<'a> From<Essence> for FluentValue<'a> {
    fn from(essence: Essence) -> Self {
        FluentValue::Number(FluentNumber::from(essence.0))
    }
}
