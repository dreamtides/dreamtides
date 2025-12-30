use ability_data::cost::Cost;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser::parser_helpers::{article, energy, word, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    choice((energy_cost(), abandon_cost()))
}

fn energy_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    energy().map(|n| Cost::Energy(Energy(n)))
}

fn abandon_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("abandon")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| Cost::AbandonCharacters(target, 1))
}
