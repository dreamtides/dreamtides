use serde::{Deserialize, Serialize};

use crate::predicate::Predicate;

/// A boolean predicate over the state of the game. Usually represented in rules
/// text by the word "if", for example "if you control 2 other warriors, draw a
/// card".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    CardsDiscardedThisTurn { count: u32 },
    CardsDrawnThisTurn { count: u32 },
    CardsInVoidCount { count: u32 },
    DissolvedThisTurn { predicate: Predicate },
    PredicateCount { count: u32, predicate: Predicate },
    ThisCharacterIsInYourVoid,
}
