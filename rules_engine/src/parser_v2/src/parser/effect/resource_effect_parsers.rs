use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Points;

use crate::parser::parser_helpers::{period, points, word, words, ParserExtra, ParserInput};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((lose_points(), enemy_gains_points())).boxed()
}

pub fn lose_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    word("you")
        .ignore_then(word("lose"))
        .ignore_then(points())
        .then_ignore(period())
        .map(|n| StandardEffect::LosePoints { loses: Points(n) })
}

pub fn enemy_gains_points<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["the", "opponent"])
        .ignore_then(word("gains"))
        .ignore_then(points())
        .then_ignore(period())
        .map(|count| StandardEffect::EnemyGainsPoints { count })
}
