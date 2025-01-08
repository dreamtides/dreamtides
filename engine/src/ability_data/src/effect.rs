use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::predicate::{CardPredicate, Predicate};

/// Represents a mutation to the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Effect(GameEffect),
    EffectList(EffectList),
}

/// Provides a sequence of effects to apply, as well as modifiers which affect
/// how those effects are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectList {
    /// Sequences of effects to apply in the provided order, usually written as
    /// complete sentences or separated by the words "then" or "and" to
    /// indicate order.
    pub effects: Vec<GameEffect>,

    /// True if this is an effect which the controller may choose to apply,
    /// usually prefixed with "You may..."
    pub optional: bool,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

/// Effects are the primary way in which cards modify the game state. This can
/// be as part of the resolution of an event card, or via the effect text of a
/// triggered or activated ability on a character card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEffect {
    DiscardCards {
        count: u64,
    },
    DrawCards {
        count: u64,
    },
    DrawMatchingCard {
        predicate: CardPredicate,
    },
    DissolveCharacter {
        target: Predicate,
    },
    GainsAegisThisTurn {
        target: Predicate,
    },
    GainsSpark {
        target: Predicate,
        gained: Spark,
    },
    TargetGainsSparkUntilYourNextMainPhaseForEach {
        target: Predicate,
        gained: Spark,
        for_each: Predicate,
    },
    GainEnergy {
        gained: Energy,
    },
}
