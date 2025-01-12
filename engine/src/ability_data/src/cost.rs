use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::collection_expression::CollectionExpression;
use crate::predicate::{CardPredicate, Predicate};

/// Any action a player must take in order to play a card or activate an
/// ability, such as paying energy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cost {
    AbandonCharacters(Predicate, u32),
    AbandonCharactersCount { target: Predicate, count: CollectionExpression },
    AbandonDreamscapes(u32),
    AbandonACharacterOrDiscardACard,
    BanishAllCardsFromYourVoid,
    BanishCardsFromEnemyVoid(u32),
    BanishCardsFromYourVoid(u32),
    BanishFromHand(Predicate),
    DiscardCards(CardPredicate, u32),
    DiscardHand,
    Energy(Energy),
    NoCost,
}
