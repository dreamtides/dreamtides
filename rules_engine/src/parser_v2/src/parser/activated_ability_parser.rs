use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    colon, comma, directive, word, words, ParserExtra, ParserInput,
};
use crate::parser::{cost_parser, effect_parser};

pub fn activated_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, ActivatedAbility, ParserExtra<'a>> + Clone {
    directive("fast")
        .or_not()
        .then_ignore(word("--").or_not())
        .then(
            choice((
                cost_parser::cost_parser().map(Some),
                words(&["once", "per", "turn"]).to(None),
            ))
            .boxed()
            .separated_by(comma())
            .at_least(1)
            .collect::<Vec<_>>(),
        )
        .then_ignore(colon())
        .then(effect_parser::effect_or_compound_parser())
        .map(|((is_fast, costs_and_options), effect)| {
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
                options: if is_fast.is_some() || once_per_turn {
                    Some(ActivatedAbilityOptions {
                        is_fast: is_fast.is_some(),
                        is_multi: !once_per_turn,
                    })
                } else {
                    None
                },
            }
        })
}
