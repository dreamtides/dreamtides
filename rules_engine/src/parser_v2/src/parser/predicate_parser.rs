use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{directive, subtype, word, words, ParserExtra, ParserInput};

pub fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((specific_predicates(), general_predicates())).boxed()
}

/// Predicate parser that excludes bare subtype.
///
/// Use this after an article to reject patterns like `a {subtype}` which
/// should use `{a-subtype}` instead.
pub fn predicate_parser_without_bare_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((specific_predicates(), general_predicates_without_bare_subtype())).boxed()
}

fn specific_predicates<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((
        reference_predicates(),
        another_parser(),
        choice((enemy_or_ally_parser(), non_subtype_enemy_parser(), allied_parser())).boxed(),
        choice((enemy_parser(), ally_parser())).boxed(),
    ))
    .boxed()
}

fn general_predicates<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((
        played_card_predicate(),
        any_fast_card_parser(),
        choice((your_void_parser(), enemy_void_parser())).boxed(),
        any_card_predicate_parser(),
        any_basic_predicates(),
    ))
    .boxed()
}

fn general_predicates_without_bare_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        played_card_predicate(),
        any_fast_card_parser(),
        choice((your_void_parser(), enemy_void_parser())).boxed(),
        any_card_predicate_parser_without_bare_subtype(),
        any_basic_predicates(),
    ))
    .boxed()
}

fn any_basic_predicates<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((any_character_parser(), any_event_parser(), any_card_parser())).boxed()
}

fn reference_predicates<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((this_parser(), that_parser(), it_parser(), them_parser())).boxed()
}

fn this_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["this", "character"]).to(Predicate::This),
        words(&["this", "event"]).to(Predicate::This),
        words(&["this", "card"]).to(Predicate::This),
    ))
}

fn it_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("it").to(Predicate::It)
}

fn that_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((words(&["that", "character"]), words(&["that", "event"]), words(&["that", "card"])))
        .to(Predicate::That)
}

fn them_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("them").to(Predicate::Them)
}

fn non_subtype_enemy_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("non-")
        .ignore_then(subtype())
        .then_ignore(word("enemy"))
        .map(|subtype| Predicate::Enemy(CardPredicate::NotCharacterType(subtype)))
}

fn enemy_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("enemy")
        .ignore_then(card_predicate_parser::parser().or_not())
        .map(|pred| Predicate::Enemy(pred.unwrap_or(CardPredicate::Character)))
}

fn allied_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("allied").ignore_then(card_predicate_parser::parser()).map(Predicate::Another)
}

fn ally_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((word("ally"), word("allies")))
        .ignore_then(card_predicate_parser::parser().or_not())
        .map(|predicate| Predicate::Another(predicate.unwrap_or(CardPredicate::Character)))
}

fn enemy_or_ally_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    words(&["enemy", "or", "ally"]).to(Predicate::Any(CardPredicate::Character))
}

fn any_card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("card").map(|_| Predicate::Any(CardPredicate::Card))
}

fn any_character_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    word("character").map(|_| Predicate::Any(CardPredicate::Character))
}

fn any_event_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("event").map(|_| Predicate::Any(CardPredicate::Event))
}

fn any_fast_card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    directive("fast")
        .ignore_then(card_predicate_parser::parser())
        .map(|target| Predicate::Any(CardPredicate::Fast { target: Box::new(target) }))
}

fn played_card_predicate<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("played").ignore_then(card_predicate_parser::parser()).map(Predicate::Any)
}

fn any_card_predicate_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser().map(Predicate::Any)
}

fn any_card_predicate_parser_without_bare_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser_without_bare_subtype().map(Predicate::Any)
}

fn another_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("another").ignore_then(card_predicate_parser::parser()).map(Predicate::Another)
}

fn your_void_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser()
        .then_ignore(words(&["in", "your", "void"]))
        .map(Predicate::YourVoid)
}

fn enemy_void_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    card_predicate_parser::parser()
        .then_ignore(words(&["in", "the", "opponent's", "void"]))
        .map(Predicate::EnemyVoid)
}
