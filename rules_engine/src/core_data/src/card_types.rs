use enumset::EnumSetType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Hash, Serialize, Deserialize, JsonSchema, Copy, Clone, Eq, PartialEq)]
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
    Ancient,
    Child,
    Detective,
    Explorer,
    Hacker,
    Mage,
    Monster,
    Musician,
    Outsider,
    Renegade,
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
