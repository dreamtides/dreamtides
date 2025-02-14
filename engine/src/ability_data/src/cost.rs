use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::collection_expression::CollectionExpression;
use crate::predicate::{CardPredicate, Predicate};

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cost {
    AbandonCharacters(Predicate, u64),
    AbandonCharactersCount { target: Predicate, count: CollectionExpression },
    AbandonDreamscapes(u64),
    AbandonACharacterOrDiscardACard,
    BanishAllCardsFromYourVoid,
    BanishCardsFromEnemyVoid(u64),
    BanishCardsFromYourVoid(u64),
    BanishFromHand(Predicate),
    DiscardCards(CardPredicate, u64),
    DiscardHand,
    Energy(Energy),
    NoCost,
}
