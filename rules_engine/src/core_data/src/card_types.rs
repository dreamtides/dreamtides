use enumset::EnumSetType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Hash, Serialize, Deserialize, JsonSchema, Copy, Clone, Eq, PartialEq)]
pub enum CardType {
    Character(CharacterType),
    Event,
    Dreamsign,
    Enemy,
    Dreamwell,
}

#[derive(
    Debug, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema, EnumSetType, Display,
)]
pub enum CharacterType {
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
}
