use ability_data::condition::Condition;
use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;
use core_data::card_types::CardSubtype;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{
    comma, count_allied_subtype, word, words, ParserExtra, ParserInput,
};

pub fn condition_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone
{
    choice((dissolved_this_turn(), with_count_allied_subtype())).boxed()
}

fn with_count_allied_subtype<'a>(
) -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone {
    word("with").ignore_then(count_allied_subtype()).then_ignore(comma()).map(|(count, subtype)| {
        Condition::PredicateCount { count, predicate: allied_subtype_predicate(subtype) }
    })
}

fn allied_subtype_predicate(subtype: CardSubtype) -> Predicate {
    Predicate::Another(CardPredicate::CharacterType(subtype))
}

fn dissolved_this_turn<'a>() -> impl Parser<'a, ParserInput<'a>, Condition, ParserExtra<'a>> + Clone
{
    word("a")
        .ignore_then(card_predicate_parser::parser())
        .then_ignore(words(&["dissolved", "this", "turn"]))
        .map(|predicate| Condition::DissolvedThisTurn { predicate: Predicate::Any(predicate) })
}
