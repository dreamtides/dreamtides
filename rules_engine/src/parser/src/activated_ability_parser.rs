use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use chumsky::Parser;
use chumsky::prelude::*;

use crate::parser_utils::{ErrorType, phrase};
use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, ErrorType<'a>> {
    let modifiers = phrase("{fa}")
        .to(Some(ActivatedAbilityOptions { is_fast: true, is_multi: false }))
        .or(phrase("{fma}").to(Some(ActivatedAbilityOptions { is_fast: true, is_multi: true })))
        .or(phrase("{ma}").to(Some(ActivatedAbilityOptions { is_fast: false, is_multi: true })))
        .or(phrase("{a}").to(None))
        .boxed();

    let costs = cost_parser::parser().separated_by(phrase(",")).collect::<Vec<_>>();

    modifiers
        .then(costs)
        .then_ignore(phrase(":"))
        .then(effect_parser::effect())
        .map(|((options, costs), effect)| ActivatedAbility { costs, effect, options })
}
