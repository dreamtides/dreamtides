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

impl CardSubtype {
    pub fn from_variable(variable: &str) -> Option<CardSubtype> {
        match variable {
            "ancient" => Some(CardSubtype::Ancient),
            "child" => Some(CardSubtype::Child),
            "detective" => Some(CardSubtype::Detective),
            "explorer" => Some(CardSubtype::Explorer),
            "hacker" => Some(CardSubtype::Hacker),
            "mage" => Some(CardSubtype::Mage),
            "monster" => Some(CardSubtype::Monster),
            "musician" => Some(CardSubtype::Musician),
            "outsider" => Some(CardSubtype::Outsider),
            "renegade" => Some(CardSubtype::Renegade),
            "spirit-animal" => Some(CardSubtype::SpiritAnimal),
            "super" => Some(CardSubtype::Super),
            "survivor" => Some(CardSubtype::Survivor),
            "synth" => Some(CardSubtype::Synth),
            "tinkerer" => Some(CardSubtype::Tinkerer),
            "trooper" => Some(CardSubtype::Trooper),
            "visionary" => Some(CardSubtype::Visionary),
            "visitor" => Some(CardSubtype::Visitor),
            "warrior" => Some(CardSubtype::Warrior),
            "enigma" => Some(CardSubtype::Enigma),
            _ => None,
        }
    }
}
