use ability_data::ability::{Ability, EventAbility};
use ability_data::triggered_ability::TriggeredAbility;
use chumsky::span::{SimpleSpan, Span};

use crate::builder::parser_spans::{
    SpannedAbility, SpannedActivatedAbility, SpannedEffect, SpannedEventAbility, SpannedText,
    SpannedTriggeredAbility,
};
use crate::lexer::lexer_token::Token;
use crate::lexer::lexer_tokenize::LexResult;

pub fn build_spanned_ability(ability: &Ability, lex_result: &LexResult) -> Option<SpannedAbility> {
    match ability {
        Ability::Event(event) => build_spanned_event(event, lex_result),
        Ability::Static(_) => {
            let full_span = if lex_result.tokens.is_empty() {
                SimpleSpan::new((), 0..lex_result.original.len())
            } else {
                let start = lex_result.tokens.first()?.1.start();
                let end = lex_result.tokens.last()?.1.end();
                SimpleSpan::new((), start..end)
            };
            Some(SpannedAbility::Static {
                text: SpannedText::new(lex_result.original.clone(), full_span),
            })
        }
        Ability::Triggered(triggered) => build_spanned_triggered(triggered, lex_result),
        Ability::Named(_) => {
            let full_span = if lex_result.tokens.is_empty() {
                SimpleSpan::new((), 0..lex_result.original.len())
            } else {
                let start = lex_result.tokens.first()?.1.start();
                let end = lex_result.tokens.last()?.1.end();
                SimpleSpan::new((), start..end)
            };
            Some(SpannedAbility::Named {
                name: SpannedText::new(lex_result.original.clone(), full_span),
            })
        }
        Ability::Activated(_) => {
            let colon_idx =
                lex_result.tokens.iter().position(|(t, _)| matches!(t, Token::Colon))?;

            let cost_end = lex_result.tokens[colon_idx].1.start();
            let cost_span = SimpleSpan::new((), 0..cost_end);

            let effect_start = lex_result.tokens[colon_idx].1.end();
            let effect_end = lex_result.tokens.last()?.1.end();
            let effect_span = SimpleSpan::new((), effect_start..effect_end);

            Some(SpannedAbility::Activated(SpannedActivatedAbility {
                cost: SpannedText::new(
                    lex_result.original[cost_span.into_range()].to_string(),
                    cost_span,
                ),
                effect: SpannedEffect::Effect(SpannedText::new(
                    lex_result.original[effect_span.into_range()].to_string(),
                    effect_span,
                )),
            }))
        }
    }
}

fn build_spanned_event(event: &EventAbility, lex_result: &LexResult) -> Option<SpannedAbility> {
    let additional_cost = if event.additional_cost.is_some() {
        let colon_idx = lex_result.tokens.iter().position(|(t, _)| matches!(t, Token::Colon))?;
        let cost_end = lex_result.tokens[colon_idx].1.start();
        let cost_span = SimpleSpan::new((), 0..cost_end);
        Some(SpannedText::new(lex_result.original[cost_span.into_range()].to_string(), cost_span))
    } else {
        None
    };

    let effect_start = if let Some(ref cost) = additional_cost { cost.span.end() + 1 } else { 0 };
    let effect_end = lex_result.tokens.last()?.1.end();
    let effect_span = SimpleSpan::new((), effect_start..effect_end);

    Some(SpannedAbility::Event(SpannedEventAbility {
        additional_cost,
        effect: SpannedEffect::Effect(SpannedText::new(
            lex_result.original[effect_span.into_range()].trim().to_string(),
            effect_span,
        )),
    }))
}

fn build_spanned_triggered(
    triggered: &TriggeredAbility,
    lex_result: &LexResult,
) -> Option<SpannedAbility> {
    let is_once_per_turn =
        triggered.options.as_ref().map(|opts| opts.once_per_turn).unwrap_or(false);

    let once_per_turn = if is_once_per_turn {
        let once_span = SimpleSpan::new((), 0..13);
        Some(SpannedText::new("Once per turn".to_string(), once_span))
    } else {
        None
    };

    let trigger_start = if once_per_turn.is_some() { 15 } else { 0 };

    let is_keyword_trigger =
        matches!(triggered.trigger, ability_data::trigger_event::TriggerEvent::Keywords(_));

    let (trigger_end, effect_start) = if is_keyword_trigger {
        let first_directive_idx = lex_result
            .tokens
            .iter()
            .position(|(t, s)| matches!(t, Token::Directive(_)) && s.start() >= trigger_start)?;
        let trigger_end = lex_result.tokens[first_directive_idx].1.end();
        let effect_start = trigger_end + 1;
        (trigger_end, effect_start)
    } else {
        let comma_or_colon_idx = lex_result.tokens.iter().position(|(t, s)| {
            matches!(t, Token::Comma | Token::Colon) && s.start() >= trigger_start
        })?;
        let trigger_end = lex_result.tokens[comma_or_colon_idx].1.start();
        let effect_start = lex_result.tokens[comma_or_colon_idx].1.end();
        (trigger_end, effect_start)
    };

    let trigger_span = SimpleSpan::new((), trigger_start..trigger_end);
    let effect_end = lex_result.tokens.last()?.1.end();
    let effect_span = SimpleSpan::new((), effect_start..effect_end);

    Some(SpannedAbility::Triggered(SpannedTriggeredAbility {
        once_per_turn,
        trigger: SpannedText::new(
            lex_result.original[trigger_span.into_range()].to_string(),
            trigger_span,
        ),
        effect: SpannedEffect::Effect(SpannedText::new(
            lex_result.original[effect_span.into_range()].trim().to_string(),
            effect_span,
        )),
    }))
}
