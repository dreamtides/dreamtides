use ability_data::triggered_ability::{TriggeredAbility, TriggeredAbilityOptions};
use chumsky::prelude::*;

use crate::parser::parser_helpers::{comma, words, ParserExtra, ParserInput};
use crate::parser::{effect_parser, trigger_parser};

pub fn triggered_ability_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggeredAbility, ParserExtra<'a>> + Clone {
    choice((once_per_turn_triggered(), simple_triggered())).boxed()
}

fn once_per_turn_triggered<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggeredAbility, ParserExtra<'a>> + Clone {
    words(&["once", "per", "turn"])
        .ignore_then(comma())
        .ignore_then(trigger_parser::trigger_event_parser())
        .then(effect_parser::effect_or_compound_parser())
        .map(|(trigger, effect)| TriggeredAbility {
            trigger,
            effect,
            options: Some(TriggeredAbilityOptions {
                once_per_turn: true,
                until_end_of_turn: false,
            }),
        })
}

fn simple_triggered<'a>(
) -> impl Parser<'a, ParserInput<'a>, TriggeredAbility, ParserExtra<'a>> + Clone {
    trigger_parser::trigger_event_parser()
        .then(effect_parser::effect_or_compound_parser())
        .map(|(trigger, effect)| TriggeredAbility { trigger, effect, options: None })
}
