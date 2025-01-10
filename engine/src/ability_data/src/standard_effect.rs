use core_data::numerics::{Energy, Points, Spark};
use serde::{Deserialize, Serialize};

use crate::collection_expression::CollectionExpression;
use crate::cost::Cost;
use crate::predicate::{CardPredicate, Predicate};
use crate::quantity_expression::QuantityExpression;
use crate::triggered_ability::TriggeredAbility;

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
    BanishThenMaterializeCount { target: Predicate, count: CollectionExpression },
    CardsInVoidGainReclaimThisTurn { count: CollectionExpression, predicate: CardPredicate },
    Copy { target: Predicate },
    CopyNextPlayed { matching: Predicate, times: Option<u64> },
    CreateTriggerUntilEndOfTurn { trigger: Box<TriggeredAbility> },
    DisableActivatedAbilitiesWhileInPlay { target: Predicate },
    DiscardCardFromEnemyHand { predicate: CardPredicate },
    DiscardCards { count: u64 },
    Discover { predicate: CardPredicate },
    DiscoverAndThenMaterialize { predicate: CardPredicate },
    DissolveCharacter { target: Predicate },
    DissolveCharactersCount { target: Predicate, count: CollectionExpression },
    DissolveCharactersQuantity { target: Predicate, quantity: QuantityExpression },
    DoubleYourEnergy,
    DrawCards { count: u64 },
    DrawMatchingCard { predicate: CardPredicate },
    DrawCardsForEach { count: u64, for_each: QuantityExpression },
    EachMatchingGainsSparkUntilNextMain { each: CardPredicate, gains: Spark },
    EachMatchingGainsSparkForEach { each: CardPredicate, gains: Spark, for_each: CardPredicate },
    EnemyGainsPoints { count: u64 },
    EnemyGainsPointsEqualToItsSpark,
    EnemyLosesPoints { count: u64 },
    Foresee { count: u64 },
    GainControl { target: Predicate },
    GainEnergy { gains: Energy },
    GainEnergyForEach { gains: Energy, for_each: Predicate },
    GainPoints { gains: Points },
    GainPointsForEach { gain: Points, for_count: QuantityExpression },
    GainsAegisThisTurn { target: Predicate },
    GainsReclaimUntilEndOfTurn { target: Predicate, cost: Option<Energy> },
    GainsSpark { target: Predicate, gains: Spark },
    GainsSparkForQuantity { target: Predicate, gains: Spark, for_quantity: QuantityExpression },
    GainsSparkUntilYourNextMainForEach { target: Predicate, gains: Spark, for_each: Predicate },
    GainTwiceThatMuchEnergyInstead,
    Kindle { amount: Spark },
    LosePoints { loses: Points },
    MaterializeCharacter { target: Predicate },
    MaterializeCharacterFromVoid { target: CardPredicate },
    MaterializeRandomFromDeck { count: u64, predicate: CardPredicate },
    Negate { target: Predicate },
    PayCost { cost: Cost },
    PutCardsFromYourDeckIntoVoid { count: u64 },
    PutOnTopOfEnemyDeck { target: Predicate },
    ReturnCharactersToHandDrawCardForEach { count: CollectionExpression },
    ReturnFromYourVoidToHand { target: Predicate },
    ReturnFromYourVoidToPlay { target: Predicate },
    ReturnToHand { target: Predicate },
    ShuffleHandAndDeckAndDraw { count: u64 },
    SpendAllEnergyDrawAndDiscard,
    TakeExtraTurn,
    TriggerJudgmentAbility { matching: Predicate, collection: CollectionExpression },
    YouWinTheGame,
}
