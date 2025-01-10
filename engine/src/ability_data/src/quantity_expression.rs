use serde::{Deserialize, Serialize};

use crate::predicate::CardPredicate;

/// Represents some quantity, such as the number of cards you have drawn in a
/// turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantityExpression {
    DiscardedThisTurn(CardPredicate),
    CardsDrawnThisTurn(CardPredicate),
    CardsInVoid(CardPredicate),
    PlayedThisTurn(CardPredicate),
}
