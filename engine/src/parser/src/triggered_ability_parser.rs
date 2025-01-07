use ability_data::triggered_ability::TriggeredAbility;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::parser_utils::{phrase, ErrorType};
use crate::{effect_parser, trigger_event_parser};

pub fn parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    choice((keyword_trigger_parser(), standard_trigger_parser()))
}

fn keyword_trigger_parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    trigger_event_parser::keyword_parser()
        .then_ignore(phrase(":"))
        .then(effect_parser::parser())
        .then_ignore(phrase("."))
        .map(|(event, effects)| TriggeredAbility::new(event, effects))
}

fn standard_trigger_parser<'a>() -> impl Parser<'a, &'a str, TriggeredAbility, ErrorType<'a>> {
    choice((phrase("whenever"), phrase("when")))
        .ignore_then(trigger_event_parser::event_parser())
        .then_ignore(phrase(","))
        .then(effect_parser::parser())
        .then_ignore(phrase("."))
        .map(|(event, effects)| TriggeredAbility::new(event, effects))
}
