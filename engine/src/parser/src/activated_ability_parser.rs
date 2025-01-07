use ability_data::activated_ability::ActivatedAbility;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{cost_parser, effect_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, ActivatedAbility, ErrorType<'a>> {
    phrase("$activated")
        .ignore_then(cost_parser::parser())
        .then_ignore(phrase(":"))
        .then(effect_parser::parser())
        .then_ignore(phrase("."))
        .map(|(cost, effect)| ActivatedAbility { cost, effect, options: None })
}
