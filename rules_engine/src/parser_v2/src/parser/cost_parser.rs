use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::parser_helpers::{
    article, cards, count, count_allies, directive, discards, energy, word, words, ParserExtra,
    ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};

pub fn cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((
        choice((
            abandon_or_discard_cost(),
            energy_cost(),
            abandon_this_character_cost(),
            abandon_cost(),
        ))
        .boxed(),
        choice((
            discard_hand_cost(),
            discard_cost(),
            banish_void_with_min_count_cost(),
            banish_from_your_void_cost(),
        ))
        .boxed(),
    ))
    .boxed()
}

pub fn banish_from_hand_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    directive("banish")
        .ignore_then(article())
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["from", "hand"]))
        .map(|predicate| Cost::BanishFromHand(Predicate::Any(predicate)))
}

pub fn banish_cards_from_your_void_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(cards())
        .then_ignore(words(&["from", "your", "void"]))
        .map(Cost::BanishCardsFromYourVoid)
}

pub fn banish_cards_from_enemy_void_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(cards())
        .then_ignore(words(&["from", "the", "opponent's", "void"]))
        .map(Cost::BanishCardsFromEnemyVoid)
}

pub fn abandon_cost_single<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    word("abandon").ignore_then(article()).ignore_then(predicate_parser::predicate_parser()).map(
        |target| Cost::AbandonCharactersCount { target, count: CollectionExpression::Exactly(1) },
    )
}

fn energy_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    energy().map(|n| Cost::Energy(Energy(n)))
}

fn abandon_this_character_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    words(&["abandon", "this", "character"]).map(|_| Cost::AbandonCharactersCount {
        target: Predicate::This,
        count: CollectionExpression::Exactly(1),
    })
}

fn abandon_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((abandon_any_number(), abandon_cost_with_count(), abandon_cost_single()))
}

fn abandon_any_number<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("abandon")
        .ignore_then(words(&["any", "number", "of"]))
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| Cost::AbandonCharactersCount {
            target,
            count: CollectionExpression::AnyNumberOf,
        })
}

fn abandon_or_discard_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    word("abandon")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .filter(|predicate| matches!(predicate, Predicate::Another(CardPredicate::Character)))
        .then_ignore(words(&["or", "discard"]))
        .then_ignore(article())
        .ignore_then(
            predicate_parser::predicate_parser()
                .filter(|predicate| matches!(predicate, Predicate::Any(CardPredicate::Card))),
        )
        .to(Cost::AbandonACharacterOrDiscardACard)
}

fn abandon_cost_with_count<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone
{
    word("abandon").ignore_then(count_allies()).map(|count| Cost::AbandonCharactersCount {
        target: Predicate::Another(CardPredicate::Character),
        count: CollectionExpression::Exactly(count),
    })
}

fn discard_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("discard")
        .ignore_then(choice((
            discards().map(|count| (CardPredicate::Card, count)),
            article().ignore_then(card_predicate_parser::parser()).map(|predicate| (predicate, 1)),
        )))
        .map(|(predicate, count)| Cost::DiscardCards(predicate, count))
}

fn discard_hand_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    words(&["discard", "your", "hand"]).to(Cost::DiscardHand)
}

fn banish_void_with_min_count_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(words(&["your", "void", "with"]))
        .ignore_then(count())
        .then_ignore(words(&["or", "more", "cards"]))
        .map(Cost::BanishAllCardsFromYourVoidWithMinCount)
}

fn banish_from_your_void_cost<'a>(
) -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    directive("banish")
        .ignore_then(predicate_parser::predicate_parser())
        .then_ignore(words(&["in", "your", "void"]))
        .map(|_| Cost::BanishCardsFromYourVoid(1))
}
