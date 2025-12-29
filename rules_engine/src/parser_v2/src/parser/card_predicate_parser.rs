use ability_data::predicate::CardPredicate;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{subtype, word, ParserExtra, ParserInput};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    choice((subtype_parser(), card_parser())).boxed()
}

fn card_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    word("card").to(CardPredicate::Card)
}

fn subtype_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone {
    subtype().map(CardPredicate::CharacterType)
}
