use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Spark;

use crate::parser::parser_helpers::{kindle_amount, ParserExtra, ParserInput};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    kindle()
}

pub fn kindle<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    kindle_amount().map(|amount| StandardEffect::Kindle { amount: Spark(amount) })
}
