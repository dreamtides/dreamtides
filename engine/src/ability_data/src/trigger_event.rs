use serde::{Deserialize, Serialize};

use crate::predicate::Predicate;

/// Describes possible game events which may cause a triggered ability to
/// trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerEvent {
    /// One or more trigger keywords
    Keywords(Vec<TriggerKeyword>),

    /// Event when a character matching a predicate is materialized.
    Materialize(Predicate),

    /// Event when the nth character matching a predicate is materialized this
    /// turn
    MaterializeNthThisTurn(Predicate, u64),

    /// Event when a card matching a predicate is played
    Play(Predicate),

    /// Event when a card matching a predicate is played from hand
    PlayFromHand(Predicate),

    /// Event when a card matching a predicate is discarded
    Discard(Predicate),

    /// Event when the end of your turn is reached
    EndOfYourTurn,

    /// Event when you gain energy
    GainEnergy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerKeyword {
    Materialized,
    Judgment,
    Dissolved,
}
