use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Spark;

use crate::parser::parser_helpers::{kindle_amount, period, ParserExtra, ParserInput};

pub fn kindle<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    kindle_amount()
        .then_ignore(period())
        .map(|amount| StandardEffect::Kindle { amount: Spark(amount) })
}
