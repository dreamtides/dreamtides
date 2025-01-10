use core_data::numerics::{Energy, Points, Spark};
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::cost::Cost;
use crate::counting_expression::CountingExpression;
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

    /// Present if this is an effect which the controller may choose to apply,
    /// usually phrased as "You may {perform effect}". A cost to perform the
    /// effect can also be specified via "You may {pay cost} to {perform
    /// effect}." templating.
    pub optional: Option<Cost>,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

impl EffectWithOptions {
    pub fn new(effect: StandardEffect) -> Self {
        Self { effect, optional: None, condition: None }
    }

    pub fn with_condition(&self, condition: Condition) -> Self {
        let mut result = self.clone();
        result.condition = Some(condition);
        result
    }

    pub fn is_optional(&self) -> bool {
        self.optional.is_some()
    }

    pub fn to_effect(self) -> Effect {
        if !self.is_optional() && self.condition.is_none() {
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
    AbandonAndGainEnergyForSpark { target: Predicate, energy_per_spark: Energy },
    AbandonAtEndOfTurn { target: Predicate },
    BanishCardsFromEnemyVoid { count: u64 },
    BanishCharacter { target: Predicate },
    BanishThenMaterialize { target: Predicate },
    BanishThenMaterializeCount { target: Predicate, count: CountingExpression },
    CreateTriggerUntilEndOfTurn { trigger: Box<TriggeredAbility> },
    DisableActivatedAbilitiesWhileInPlay { target: Predicate },
    DiscardCardFromEnemyHand { predicate: CardPredicate },
    DiscardCards { count: u64 },
    Discover { predicate: CardPredicate },
    DiscoverAndThenMaterialize { predicate: CardPredicate },
    DissolveCharacter { target: Predicate },
    DissolveCharactersCount { target: Predicate, count: CountingExpression },
    DrawCards { count: u64 },
    DrawCardsForEachAbandoned { count: u64 },
    DrawMatchingCard { predicate: CardPredicate },
    EachMatchingGainsSparkForEach { each: CardPredicate, gains: Spark, for_each: CardPredicate },
    EnemyGainsPoints { count: u64 },
    EnemyGainsPointsEqualToItsSpark,
    EnemyLosesPoints { count: u64 },
    Foresee { count: u64 },
    GainControl { target: Predicate },
    GainEnergy { gains: Energy },
    GainEnergyForEach { gains: Energy, for_each: Predicate },
    GainPoints { gains: Points },
    GainsAegisThisTurn { target: Predicate },
    GainsReclaimUntilEndOfTurn { target: Predicate },
    GainsSpark { target: Predicate, gains: Spark },
    GainsSparkUntilYourNextMainForEach { target: Predicate, gains: Spark, for_each: Predicate },
    Kindle { amount: Spark },
    LosePoints { loses: Points },
    MaterializeCharacter { target: Predicate },
    MaterializeCharacterFromVoid { target: CardPredicate },
    MaterializeRandomFromDeck { count: u64, predicate: CardPredicate },
    Negate { target: Predicate },
    PayCost { cost: Cost },
    PutOnTopOfEnemyDeck { target: Predicate },
    ReturnCharactersToHandDrawCardForEach { count: CountingExpression },
    ReturnFromYourVoidToHand { target: Predicate },
    ReturnFromYourVoidToPlay { target: Predicate },
    SpendAllEnergyDrawAndDiscard,
}
