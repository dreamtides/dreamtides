use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;

use crate::parser::card_predicate_parser;
use crate::parser::parser_helpers::{word, words, ParserExtra, ParserInput};

pub fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((this_parser(), enemy_parser(), any_card_parser())).boxed()
}

fn this_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    choice((
        words(&["this", "character"]).to(Predicate::This),
        words(&["this", "event"]).to(Predicate::This),
        words(&["this", "card"]).to(Predicate::This),
    ))
}

fn enemy_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("enemy")
        .ignore_then(card_predicate_parser::parser().or_not())
        .map(|pred| Predicate::Enemy(pred.unwrap_or(CardPredicate::Character)))
}

fn any_card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone {
    word("card").map(|_| Predicate::Any(CardPredicate::Card))
}
