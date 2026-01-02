use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{colon, comma, words, ParserExtra, ParserInput};
use crate::parser::{cost_parser, effect_parser};

pub fn activated_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, ActivatedAbility, ParserExtra<'a>> + Clone {
    choice((cost_parser::cost_parser().map(Some), words(&["once", "per", "turn"]).to(None)))
        .boxed()
        .separated_by(comma())
        .at_least(1)
        .collect::<Vec<_>>()
        .then_ignore(colon())
        .then(effect_parser::effect_or_compound_parser())
        .map(|(costs_and_options, effect)| {
            let (costs, once_per_turn) = costs_and_options.into_iter().fold(
                (Vec::new(), false),
                |(mut costs, mut once_per_turn), cost| {
                    if let Some(cost) = cost {
                        costs.push(cost);
                    } else {
                        once_per_turn = true;
                    }
                    (costs, once_per_turn)
                },
            );
            ActivatedAbility {
                costs,
                effect,
                options: once_per_turn
                    .then_some(ActivatedAbilityOptions { is_fast: false, is_multi: false }),
            }
        })
}
