use ability_data::ability::{
    DisplayedAbility, DisplayedAbilityEffect, DisplayedEventAbility, DisplayedModalEffectChoice,
};
use chumsky::span::SimpleSpan;

use crate::builder::spanned::{
    SpannedAbility, SpannedEffect, SpannedModalEffectChoice, SpannedText,
};

/// Converts a [SpannedAbility] to a [DisplayedAbility].
pub fn to_displayed_ability(original: &str, ability: &SpannedAbility) -> DisplayedAbility {
    match ability {
        SpannedAbility::Event(event) => DisplayedAbility::Event {
            event: DisplayedEventAbility {
                additional_cost: event
                    .additional_cost
                    .as_ref()
                    .map(|cost| spanned_text_to_string(original, cost)),
                effect: spanned_effect_to_displayed(original, &event.effect),
            },
        },
        SpannedAbility::Static { text } => {
            DisplayedAbility::Static { text: spanned_text_to_string(original, text) }
        }
        SpannedAbility::Activated(activated) => DisplayedAbility::Activated {
            cost: spanned_text_to_string(original, &activated.cost),
            effect: spanned_effect_to_displayed(original, &activated.effect),
        },
        SpannedAbility::Triggered(triggered) => DisplayedAbility::Triggered {
            text: format!(
                "{}{}{}",
                triggered
                    .once_per_turn
                    .as_ref()
                    .map(|opt| format!("{}, ", spanned_text_to_string(original, opt)))
                    .unwrap_or_default(),
                spanned_text_to_string(original, &triggered.trigger),
                spanned_effect_to_displayed(original, &triggered.effect).to_string(original)
            ),
        },
        SpannedAbility::Named { name } => {
            DisplayedAbility::Named { name: spanned_text_to_string(original, name) }
        }
    }
}

fn extract_text(original: &str, span: &SimpleSpan) -> String {
    original[span.into_range()].to_string()
}

fn spanned_text_to_string(original: &str, spanned: &SpannedText) -> String {
    extract_text(original, &spanned.span)
}

fn spanned_effect_to_displayed(original: &str, effect: &SpannedEffect) -> DisplayedAbilityEffect {
    match effect {
        SpannedEffect::Effect(text) => {
            DisplayedAbilityEffect::Effect(spanned_text_to_string(original, text))
        }
        SpannedEffect::Modal(choices) => DisplayedAbilityEffect::Modal(
            choices
                .iter()
                .map(|choice| spanned_modal_choice_to_displayed(original, choice))
                .collect(),
        ),
    }
}

fn spanned_modal_choice_to_displayed(
    original: &str,
    choice: &SpannedModalEffectChoice,
) -> DisplayedModalEffectChoice {
    DisplayedModalEffectChoice {
        cost: spanned_text_to_string(original, &choice.cost),
        effect: spanned_text_to_string(original, &choice.effect),
    }
}

trait DisplayedAbilityEffectExt {
    fn to_string(&self, original: &str) -> String;
}

impl DisplayedAbilityEffectExt for DisplayedAbilityEffect {
    fn to_string(&self, _original: &str) -> String {
        match self {
            DisplayedAbilityEffect::Effect(text) => text.clone(),
            DisplayedAbilityEffect::Modal(choices) => choices
                .iter()
                .map(|c| format!("{}: {}", c.cost, c.effect))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}
