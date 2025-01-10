use ability_data::activated_ability::{ActivatedAbility, ActivatedAbilityOptions};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, ErrorType<'a>> {
    let modifiers = phrase("$fastactivated")
        .to(Some(ActivatedAbilityOptions { is_fast: true, is_immediate: false, is_multi: false }))
        .or(phrase("$fastmultiactivated").to(Some(ActivatedAbilityOptions {
            is_fast: true,
            is_immediate: false,
            is_multi: true,
        })))
        .or(phrase("$multiactivated").to(Some(ActivatedAbilityOptions {
            is_fast: false,
            is_immediate: false,
            is_multi: true,
        })))
        .or(phrase("$immediate").then(phrase("$activated")).to(Some(ActivatedAbilityOptions {
            is_fast: false,
            is_immediate: true,
            is_multi: false,
        })))
        .or(phrase("$immediate").then(phrase("$multiactivated")).to(Some(
            ActivatedAbilityOptions { is_fast: false, is_immediate: true, is_multi: true },
        )))
        .or(phrase("$immediate").then(phrase("$fastactivated")).to(Some(ActivatedAbilityOptions {
            is_fast: true,
            is_immediate: true,
            is_multi: false,
        })))
        .or(phrase("$immediate").then(phrase("$fastmultiactivated")).to(Some(
            ActivatedAbilityOptions { is_fast: true, is_immediate: true, is_multi: true },
        )))
        .or(phrase("$activated").to(None))
        .boxed();

    let costs = cost_parser::parser().separated_by(phrase(",")).collect::<Vec<_>>();

    modifiers
        .then(costs)
        .then_ignore(phrase(":"))
        .then(effect_parser::effect())
        .map(|((options, costs), effect)| ActivatedAbility { costs, effect, options })
        .boxed()
}
