use serde::{Deserialize, Serialize};

/// Represents possible subtypes of 'character' cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterType {
    Warrior,
    Survivor,
    SpiritAnimal,
}
