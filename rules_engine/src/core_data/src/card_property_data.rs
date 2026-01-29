use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Special,
}
