use serde::{Deserialize, Serialize};

use crate::predicate::{CardPredicate, Predicate};

/// Represents some quantity, such as the number of cards you have drawn in a
/// turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantityExpression {
    AbandonedThisTurn(CardPredicate),
    AbandonedThisWay(CardPredicate),
    CardsDrawnThisTurn(CardPredicate),
    DiscardedThisTurn(CardPredicate),
    Matching(Predicate),
    PlayedThisTurn(CardPredicate),
}
