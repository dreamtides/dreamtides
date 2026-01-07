use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::Parser;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser_utils::{ErrorType, count, number, numeric, phrase};
use crate::{card_predicate_parser, collection_expression_parser, determiner_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((numeric("{-energy-cost(e:", Energy, ")}").map(Cost::Energy), standard_cost())).boxed()
}

/// Costs written as a standard verb phrase, for example "pay $1" or "discard a
/// card".
pub fn standard_cost<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("pay one or more {e}").to(Cost::SpendOneOrMoreEnergy),
        numeric("pay {-energy-cost(e:", Energy, ")}").map(Cost::Energy),
        phrase("banish a card from your void").to(Cost::BanishCardsFromYourVoid(1)),
        numeric("banish", count, "cards from your void").map(Cost::BanishCardsFromYourVoid),
        phrase("banish a card from the enemy's void").to(Cost::BanishCardsFromEnemyVoid(1)),
        numeric("banish", count, "cards from the enemy's void").map(Cost::BanishCardsFromEnemyVoid),
        phrase("abandon a character or discard a card").to(Cost::Choice(vec![
            Cost::AbandonCharactersCount {
                target: Predicate::Another(CardPredicate::Character),
                count: CollectionExpression::Exactly(1),
            },
            Cost::DiscardCards { target: Predicate::Any(CardPredicate::Card), count: 1 },
        ])),
        phrase("abandon a dreamscape").to(Cost::AbandonDreamscapes(1)),
        numeric("abandon", count, "dreamscapes").map(Cost::AbandonDreamscapes),
        phrase("abandon").ignore_then(determiner_parser::your_action()).map(|target| {
            Cost::AbandonCharactersCount { target, count: CollectionExpression::Exactly(1) }
        }),
        abandon_characters_count(),
        phrase("discard your hand").to(Cost::DiscardHand),
        phrase("discard")
            .ignore_then(numeric("{-discarded-cards(n:", count, ")}"))
            .map(|n| Cost::DiscardCards { target: Predicate::Any(CardPredicate::Card), count: n }),
        phrase("discard a")
            .ignore_then(card_predicate_parser::parser())
            .map(|predicate| Cost::DiscardCards { target: Predicate::Any(predicate), count: 1 }),
        phrase("discard").ignore_then(number(count)).then(card_predicate_parser::parser()).map(
            |(count, predicate)| Cost::DiscardCards { target: Predicate::Any(predicate), count },
        ),
        phrase("spend any amount of energy").to(Cost::SpendOneOrMoreEnergy),
    ))
}

/// Alternate phrasing for costs, which are written in static abilities, for
/// example "You may play this event for $0 by abandoning a character".
pub fn present_participle_additional_cost<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        phrase("banishing another card from your void").to(Cost::BanishCardsFromYourVoid(1)),
        phrase("banishing all other cards from your void").to(Cost::BanishAllCardsFromYourVoid),
        phrase("banishing all cards from your void").to(Cost::BanishAllCardsFromYourVoid),
        phrase("abandoning a dreamscape").to(Cost::AbandonDreamscapes(1)),
        numeric("abandoning", count, "dreamscapes").map(Cost::AbandonDreamscapes),
        choice((phrase("abandoning a").to(1), numeric("abandoning", count, "")))
            .then(card_predicate_parser::parser())
            .map(|(n, predicate)| Cost::AbandonCharactersCount {
                target: Predicate::Your(predicate),
                count: CollectionExpression::Exactly(n),
            }),
        phrase("banishing")
            .ignore_then(determiner_parser::your_action())
            .then_ignore(phrase("from your hand"))
            .map(Cost::BanishFromHand),
        phrase("discarding")
            .ignore_then(numeric("{-discarded-cards(n:", count, ")}"))
            .map(|n| Cost::DiscardCards { target: Predicate::Any(CardPredicate::Card), count: n }),
        phrase("discarding a")
            .ignore_then(card_predicate_parser::parser())
            .map(|predicate| Cost::DiscardCards { target: Predicate::Any(predicate), count: 1 }),
        phrase("discarding").ignore_then(number(count)).then(card_predicate_parser::parser()).map(
            |(count, predicate)| Cost::DiscardCards { target: Predicate::Any(predicate), count },
        ),
    ))
}

/// Costs written as a third-person singular present-tense verb phrase, for
/// example "pays $1" or "discards a card".
pub fn third_person_singular_present_tense_cost<'a>()
-> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    choice((
        numeric("pays {-energy-cost(e:", Energy, ")}").map(Cost::Energy),
        phrase("banishes a card from their void").to(Cost::BanishCardsFromYourVoid(1)),
        numeric("banishes", count, "cards from their void").map(Cost::BanishCardsFromYourVoid),
        phrase("abandons a character or discards a card").to(Cost::Choice(vec![
            Cost::AbandonCharactersCount {
                target: Predicate::Another(CardPredicate::Character),
                count: CollectionExpression::Exactly(1),
            },
            Cost::DiscardCards { target: Predicate::Any(CardPredicate::Card), count: 1 },
        ])),
        phrase("abandons a dreamscape").to(Cost::AbandonDreamscapes(1)),
        numeric("abandons", count, "dreamscapes").map(Cost::AbandonDreamscapes),
        phrase("abandons").ignore_then(determiner_parser::your_action()).map(|target| {
            Cost::AbandonCharactersCount { target, count: CollectionExpression::Exactly(1) }
        }),
        abandon_characters_count(),
        phrase("discards their hand").to(Cost::DiscardHand),
        phrase("discards")
            .ignore_then(numeric("{-discarded-cards(n:", count, ")}"))
            .map(|n| Cost::DiscardCards { target: Predicate::Any(CardPredicate::Card), count: n }),
        phrase("discards a")
            .ignore_then(card_predicate_parser::parser())
            .map(|predicate| Cost::DiscardCards { target: Predicate::Any(predicate), count: 1 }),
        phrase("discards").ignore_then(number(count)).then(card_predicate_parser::parser()).map(
            |(count, predicate)| Cost::DiscardCards { target: Predicate::Any(predicate), count },
        ),
    ))
}

fn abandon_characters_count<'a>() -> impl Parser<'a, &'a str, Cost, ErrorType<'a>> {
    phrase("abandon")
        .ignore_then(collection_expression_parser::parser())
        .then(determiner_parser::your_action_counted_parser())
        .map(|(count, target)| Cost::AbandonCharactersCount { target, count })
}
