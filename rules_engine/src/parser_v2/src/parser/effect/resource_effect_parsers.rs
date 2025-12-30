use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Points;

use crate::parser::parser_helpers::{points, words, ParserExtra, ParserInput};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((lose_points(), enemy_gains_points())).boxed()
}

pub fn lose_points<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone
{
    words(&["you", "lose"])
        .ignore_then(points())
        .map(|n| StandardEffect::LosePoints { loses: Points(n) })
}

pub fn enemy_gains_points<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    words(&["the", "opponent", "gains"])
        .ignore_then(points())
        .map(|count| StandardEffect::EnemyGainsPoints { count })
}
