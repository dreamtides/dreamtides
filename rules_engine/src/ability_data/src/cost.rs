use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::collection_expression::CollectionExpression;
use crate::predicate::{CardPredicate, Predicate};

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cost {
    AbandonCharactersCount { target: Predicate, count: CollectionExpression },
    AbandonDreamscapes(u32),
    AbandonACharacterOrDiscardACard,
    BanishAllCardsFromYourVoid,
    BanishAllCardsFromYourVoidWithMinCount(u32),
    BanishCardsFromEnemyVoid(u32),
    BanishCardsFromYourVoid(u32),
    BanishFromHand(Predicate),
    CostList(Vec<Cost>),
    DiscardCards(CardPredicate, u32),
    DiscardHand,
    Energy(Energy),
    ReturnToHand(Predicate),
    SpendOneOrMoreEnergy,
}

impl Cost {
    /// Returns the energy cost of this cost, if it has one.
    ///
    /// If the cost is a list of costs, returns the first energy cost found.
    /// Cost lists are assumed to not contain repeated cost types.
    pub fn energy_cost(&self) -> Option<Energy> {
        match self {
            Cost::Energy(energy) => Some(*energy),
            Cost::CostList(costs) => costs.iter().find_map(Cost::energy_cost),
            _ => None,
        }
    }
}
