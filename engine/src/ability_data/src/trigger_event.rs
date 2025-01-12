use serde::{Deserialize, Serialize};

use crate::predicate::Predicate;

/// Describes possible game events which may cause a triggered ability to
/// trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerEvent {
    Abandon(Predicate),
    Banished(Predicate),
    Discard(Predicate),
    Dissolved(Predicate),
    DrawAllCardsInCopyOfDeck,
    EndOfYourTurn,
    GainEnergy,
    Keywords(Vec<TriggerKeyword>),
    Materialize(Predicate),
    MaterializeNthThisTurn(Predicate, u32),
    Play(Predicate),
    PlayFromHand(Predicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerKeyword {
    Materialized,
    Judgment,
    Dissolved,
}
