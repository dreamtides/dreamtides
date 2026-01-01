use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Spark;

use crate::parser::parser_helpers::{kindle_amount, spark, word, words, ParserExtra, ParserInput};
use crate::parser::predicate_parser;

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((kindle(), gains_spark())).boxed()
}

pub fn kindle<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    kindle_amount().map(|amount| StandardEffect::Kindle { amount: Spark(amount) })
}

fn gains_spark<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    predicate_parser::predicate_parser()
        .then_ignore(words(&["gains", "+"]))
        .then(spark())
        .then_ignore(word("spark"))
        .map(|(target, gains)| StandardEffect::GainsSpark { target, gains: Spark(gains) })
}
