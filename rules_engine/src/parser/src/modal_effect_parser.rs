use ability_data::effect::{Effect, ModalEffectChoice};
use chumsky::Parser;
use chumsky::prelude::*;
use core_data::numerics::Energy;

use crate::parser_utils::{ErrorType, numeric, phrase};

pub fn parser_with<'a, P>(effect: P) -> impl Parser<'a, &'a str, Effect, ErrorType<'a>>
where
    P: Parser<'a, &'a str, Effect, ErrorType<'a>> + Clone,
{
    phrase("{choose-one}")
        .ignore_then(mode(effect).repeated().at_least(1).collect::<Vec<_>>())
        .map(Effect::Modal)
}

fn mode<'a, P>(effect: P) -> impl Parser<'a, &'a str, ModalEffectChoice, ErrorType<'a>>
where
    P: Parser<'a, &'a str, Effect, ErrorType<'a>> + Clone,
{
    phrase("{mode}")
        .ignore_then(numeric("{-energy-cost(e:", Energy, ")}"))
        .then_ignore(just(":"))
        .then(effect)
        .then_ignore(phrase("{end-mode}"))
        .map(|(energy_cost, effect)| ModalEffectChoice { energy_cost, effect })
}
