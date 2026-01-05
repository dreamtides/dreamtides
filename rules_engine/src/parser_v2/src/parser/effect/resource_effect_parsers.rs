use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::StandardStaticAbility;
use chumsky::prelude::*;
use core_data::numerics::Points;

use crate::parser::parser_helpers::{directive, number, points, words, ParserExtra, ParserInput};

pub fn parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        multiply_energy_gain_from_card_effects(),
        multiply_card_draw_from_card_effects(),
        multiply_your_energy(),
        lose_points(),
        enemy_gains_points(),
    ))
    .boxed()
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

pub fn multiply_energy_gain_from_card_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    number()
        .then_ignore(words(&["the", "amount", "of"]))
        .then_ignore(directive("energy-symbol"))
        .then_ignore(words(&["you", "gain", "from", "card", "effects", "this", "turn"]))
        .map(|multiplier| StandardEffect::CreateStaticAbilityUntilEndOfTurn {
            ability: Box::new(StandardStaticAbility::MultiplyEnergyGainFromCardEffects {
                multiplier,
            }),
        })
}

pub fn multiply_card_draw_from_card_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    number()
        .then_ignore(words(&[
            "the", "number", "of", "cards", "you", "draw", "from", "card", "effects", "this",
            "turn",
        ]))
        .map(|multiplier| StandardEffect::CreateStaticAbilityUntilEndOfTurn {
            ability: Box::new(StandardStaticAbility::MultiplyCardDrawFromCardEffects {
                multiplier,
            }),
        })
}

pub fn multiply_your_energy<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    number()
        .then_ignore(words(&["the", "amount", "of"]))
        .then_ignore(directive("energy-symbol"))
        .then_ignore(words(&["you", "have"]))
        .map(|multiplier| StandardEffect::MultiplyYourEnergy { multiplier })
}
