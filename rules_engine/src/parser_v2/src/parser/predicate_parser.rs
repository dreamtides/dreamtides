use ability_data::predicate::{CardPredicate, Predicate};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{word, ParserExtra, ParserInput};

pub fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> + Clone
{
    choice((
        word("this").ignore_then(word("character")).to(Predicate::This),
        word("an").ignore_then(word("ally")).to(Predicate::Your(CardPredicate::Character)),
        word("a").ignore_then(word("card")).to(Predicate::Any(CardPredicate::Card)),
        word("a").ignore_then(word("character")).to(Predicate::Any(CardPredicate::Character)),
        word("an").ignore_then(word("event")).to(Predicate::Any(CardPredicate::Event)),
    ))
    .boxed()
}
