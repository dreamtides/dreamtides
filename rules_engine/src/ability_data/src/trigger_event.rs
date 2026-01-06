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
    LeavesPlay(Predicate),
    PutIntoVoid(Predicate),
    DrawAllCardsInCopyOfDeck,
    DrawCardsInTurn(u32),
    EndOfYourTurn,
    GainEnergy,
    Keywords(Vec<TriggerKeyword>),
    Materialize(Predicate),
    MaterializeNthThisTurn(Predicate, u32),
    Play(Predicate),
    PlayCardsInTurn(u32),
    PlayDuringTurn(Predicate, PlayerTurn),
    PlayFromHand(Predicate),
    OpponentPlays(Predicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerKeyword {
    Materialized,
    Judgment,
    Dissolved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerTurn {
    YourTurn,
    EnemyTurn,
}
