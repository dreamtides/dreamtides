use ability_data::cost::Cost;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{article, word, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    abandon_cost()
}

fn abandon_cost<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> + Clone {
    word("abandon")
        .ignore_then(article())
        .ignore_then(predicate_parser::predicate_parser())
        .map(|target| Cost::AbandonCharacters(target, 1))
}
