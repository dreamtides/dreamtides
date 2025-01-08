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

    /// Event when a card matching a predicate is played
    Play(Predicate),

    /// Event when a card matching a predicate is discarded
    Discard(Predicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerKeyword {
    Materialized,
    Judgment,
    Dissolved,
}
