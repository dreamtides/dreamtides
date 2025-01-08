use ability_data::triggered_ability::{TriggeredAbility, TriggeredAbilityOptions};
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{effect_parser, trigger_event_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    choice((keyword_trigger_parser(), standard_trigger_parser())).boxed()
}

fn keyword_trigger_parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    trigger_event_parser::keyword_parser()
        .then_ignore(phrase(":"))
        .then(effect_parser::effect())
        .map(|(trigger, effect)| TriggeredAbility { trigger, effect, options: None })
        .boxed()
}

pub fn standard_trigger_parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    phrase("once per turn,")
        .or_not()
        .then_ignore(choice((phrase("whenever"), phrase("when"))))
        .then(trigger_event_parser::event_parser())
        .then_ignore(phrase(","))
        .then(effect_parser::effect())
        .map(move |((once_per_turn, trigger), effect)| TriggeredAbility {
            trigger,
            effect,
            options: once_per_turn
                .map(|_| TriggeredAbilityOptions { once_per_turn: true, until_end_of_turn: false }),
        })
        .boxed()
}
