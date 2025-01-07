use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use ability_data::cost::Cost;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, ErrorType<'a>> {
    let fast_indicator = phrase("$fastactivated").to(true).or(phrase("$activated").to(false));
    fast_indicator
        .then(cost_parser::parser().or_not())
        .then_ignore(phrase(":"))
        .then(effect_parser::parser())
        .then_ignore(phrase("."))
        .map(|((is_fast, cost), effect)| {
            let options = if is_fast {
                Some(ActivatedAbilityOptions {
                    is_fast: true,
                    is_immediate: false,
                    is_multi: false,
                })
            } else {
                None
            };
            ActivatedAbility { cost: cost.unwrap_or(Cost::None), effect, options }
        })
}
