use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::predicate::{CardPredicate, Predicate};
use crate::triggered_ability::TriggeredAbility;

/// Represents a mutation to the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Effect(StandardEffect),
    WithOptions(EffectWithOptions),
    List(Vec<EffectWithOptions>),
}

/// Provides an effect along with configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectWithOptions {
    /// Effect to apply
    pub effect: StandardEffect,

    /// True if this is an effect which the controller may choose to apply,
    /// usually prefixed with "You may..."
    pub optional: bool,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

impl EffectWithOptions {
    pub fn new(effect: StandardEffect) -> Self {
        Self { effect, optional: false, condition: None }
    }

    pub fn with_condition(&self, condition: Condition) -> Self {
        let mut result = self.clone();
        result.condition = Some(condition);
        result
    }

    pub fn to_effect(self) -> Effect {
        if !self.optional && self.condition.is_none() {
            Effect::Effect(self.effect)
        } else {
            Effect::WithOptions(self)
        }
    }
}

/// Effects are the primary way in which cards modify the game state. This can
/// be as part of the resolution of an event card, or via the effect text of a
/// triggered or activated ability on a character card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardEffect {
    CreateTriggerUntilEndOfTurn {
        trigger: Box<TriggeredAbility>,
    },
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
    DisableActivatedAbilitiesWhileInPlay {
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
    GainEnergyForEach {
        gained: Energy,
        for_each: Predicate,
    },
    BanishCardsFromEnemyVoid {
        count: u64,
    },
    AbandonAndGainEnergyForSpark {
        target: Predicate,
        energy_per_spark: Energy,
    },
}
