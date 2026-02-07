use std::str::FromStr;

use enumset::EnumSetType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Debug, Hash, Serialize, Deserialize, JsonSchema, Copy, Clone, Eq, PartialEq, EnumString,
)]
pub enum CardType {
    Character,
    Event,
    Dreamsign,
    Dreamcaller,
    Dreamwell,
}

#[derive(
    Debug,
    Hash,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    JsonSchema,
    EnumSetType,
    EnumString,
    Display,
)]
pub enum CardSubtype {
    Agent,
    Ancient,
    Avatar,
    Child,
    Detective,
    Explorer,
    Guide,
    Hacker,
    Mage,
    Monster,
    Musician,
    Outsider,
    Renegade,
    Robot,
    #[strum(to_string = "Spirit Animal", serialize = "SpiritAnimal")]
    SpiritAnimal,
    Super,
    Survivor,
    Synth,
    Tinkerer,
    Trooper,
    Visionary,
    Visitor,
    Warrior,

    #[serde(other)]
    Enigma,
}

impl CardSubtype {
    /// Parses a PascalCase variable string into a CardSubtype.
    pub fn from_variable(variable: &str) -> Option<CardSubtype> {
        Self::from_str(variable).ok()
    }
}
