use ability_data::activated_ability::ActivatedAbility;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{colon, ParserExtra, ParserInput};
use crate::parser::{cost_parser, effect_parser};

pub fn activated_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, ActivatedAbility, ParserExtra<'a>> + Clone {
    cost_parser::cost_parser()
        .then_ignore(colon())
        .then(effect_parser::effect_or_compound_parser())
        .map(|(cost, effect)| ActivatedAbility { costs: vec![cost], effect, options: None })
}
