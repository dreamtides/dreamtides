use std::collections::{BTreeMap, HashMap};

use ability_data::ability::Ability;
use ability_data::activated_ability::ActivatedAbility;
use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use ability_data::named_ability::NamedAbility;
use ability_data::trigger_event::TriggerEvent;
use ability_data::triggered_ability::TriggeredAbility;
use strings::strings;

use crate::serializer::effect_serializer::AbilityContext;
use crate::serializer::{
    cost_serializer, effect_serializer, static_ability_serializer, trigger_serializer,
};

/// Result of serializing an ability into displayable text.
pub struct SerializedAbility {
    /// The rendered rules text for this ability.
    pub text: String,
}

/// Serializes an ability into its rules text and variable bindings.
pub fn serialize_ability(ability: &Ability) -> SerializedAbility {
    let text = match ability {
        Ability::Triggered(triggered) => serialize_triggered(triggered),
        Ability::Event(event) => {
            strings::capitalized_sentence(effect_serializer::serialize_effect(&event.effect))
                .to_string()
        }
        Ability::Activated(activated) => serialize_activated(activated),
        Ability::Named(named) => serialize_named_ability(named),
        Ability::Static(static_ability) => strings::capitalized_sentence(
            static_ability_serializer::serialize_static_ability(static_ability),
        )
        .to_string(),
    };
    SerializedAbility { text: guard_resolve_rlf(&text) }
}

/// Serializes just the effect portion of an ability, without any costs.
///
/// For event/activated abilities, returns only the effect text.
/// For triggered/static/named abilities, returns the full ability text.
pub fn serialize_ability_effect(ability: &Ability) -> SerializedAbility {
    let text = match ability {
        Ability::Event(event) => {
            strings::capitalized_sentence(effect_serializer::serialize_effect(&event.effect))
                .to_string()
        }
        Ability::Activated(activated) => {
            strings::capitalized_sentence(effect_serializer::serialize_effect(&activated.effect))
                .to_string()
        }
        _ => return serialize_ability(ability),
    };
    SerializedAbility { text: guard_resolve_rlf(&text) }
}

/// Extracts and serializes each modal effect choice from a list of abilities.
///
/// Returns a map from choice index to serialized effect text.
pub fn serialize_modal_choices(
    abilities: &[Ability],
) -> BTreeMap<ModelEffectChoiceIndex, SerializedAbility> {
    let mut result = BTreeMap::new();
    let mut current_index = 0usize;
    for ability in abilities {
        let effect = match ability {
            Ability::Event(event) => Some(&event.effect),
            Ability::Activated(activated) => Some(&activated.effect),
            _ => None,
        };
        if let Some(Effect::Modal(choices)) = effect {
            for choice in choices {
                let text = strings::capitalized_sentence(effect_serializer::serialize_effect(
                    &choice.effect,
                ))
                .to_string();
                result.insert(ModelEffectChoiceIndex(current_index), SerializedAbility {
                    text: guard_resolve_rlf(&text),
                });
                current_index += 1;
            }
        }
    }
    result
}

/// Assembles a triggered ability using phrase-based composition.
fn serialize_triggered(triggered: &TriggeredAbility) -> String {
    let has_once_per_turn = triggered.options.as_ref().map(|o| o.once_per_turn).unwrap_or(false);
    let has_until_end_of_turn =
        triggered.options.as_ref().map(|o| o.until_end_of_turn).unwrap_or(false);
    let has_prefix = has_once_per_turn || has_until_end_of_turn;
    let trigger = trigger_serializer::serialize_trigger_event(&triggered.trigger);
    let effect = effect_serializer::serialize_effect_with_context(
        &triggered.effect,
        AbilityContext::Triggered,
    );
    let is_keyword_trigger = matches!(triggered.trigger, TriggerEvent::Keywords(_));
    let prefix = build_trigger_prefix(has_until_end_of_turn, has_once_per_turn);
    match (has_prefix, is_keyword_trigger) {
        (true, true) => {
            strings::prefixed_keyword_triggered_ability(prefix, trigger, effect).to_string()
        }
        (true, false) => strings::prefixed_triggered_ability(prefix, trigger, effect).to_string(),
        (false, true) => strings::keyword_triggered_ability(trigger, effect).to_string(),
        (false, false) => strings::triggered_ability(trigger, effect).to_string(),
    }
}

/// Builds the combined prefix string for trigger modifiers.
fn build_trigger_prefix(has_until_end_of_turn: bool, has_once_per_turn: bool) -> String {
    let mut prefix = String::new();
    if has_until_end_of_turn {
        prefix.push_str(&strings::until_end_of_turn_prefix().to_string());
    }
    if has_once_per_turn {
        prefix.push_str(&strings::once_per_turn_prefix().to_string());
    }
    prefix
}

/// Assembles an activated ability using phrase-based composition.
fn serialize_activated(activated: &ActivatedAbility) -> String {
    let is_fast = activated.options.as_ref().is_some_and(|options| options.is_fast);
    let has_once_per_turn = activated.options.as_ref().is_some_and(|options| !options.is_multi);
    let costs = activated
        .costs
        .iter()
        .map(|cost| {
            strings::capitalized_sentence(cost_serializer::serialize_cost(cost)).to_string()
        })
        .collect::<Vec<_>>()
        .join(&strings::activated_cost_separator().to_string());
    let effect = effect_serializer::serialize_effect_with_context(
        &activated.effect,
        AbilityContext::Triggered,
    );
    match (is_fast, has_once_per_turn) {
        (true, true) => strings::fast_activated_ability_once_per_turn(costs, effect).to_string(),
        (true, false) => strings::fast_activated_ability(costs, effect).to_string(),
        (false, true) => strings::activated_ability_once_per_turn(costs, effect).to_string(),
        (false, false) => strings::activated_ability(costs, effect).to_string(),
    }
}

/// Compatibility guard: resolves any remaining RLF phrase references in a
/// template string. Retained until the parity gate task confirms that
/// phrase-based assembly produces identical output for all cards, at which
/// point this function and its call sites can be deleted.
fn guard_resolve_rlf(template: &str) -> String {
    strings::register_source_phrases();
    rlf::with_locale(|locale| {
        let resolved = locale
            .eval_str(template, HashMap::new())
            .unwrap_or_else(|e| panic!("Error resolving RLF template {template:?}: {e}"))
            .to_string();
        debug_assert_eq!(
            resolved, template,
            "Parity violation: resolve_rlf changed output.\n  Before: {template:?}\n  After:  {resolved:?}"
        );
        resolved
    })
}

/// Serializes a named ability into its display text.
fn serialize_named_ability(named: &NamedAbility) -> String {
    match named {
        NamedAbility::Reclaim(cost) => {
            if let Some(energy_cost) = cost {
                strings::capitalized_sentence(strings::reclaim_for_cost(energy_cost.0)).to_string()
            } else {
                strings::capitalized_sentence(strings::reclaim()).to_string()
            }
        }
        NamedAbility::ReclaimForCost(cost) => strings::reclaim_with_cost(
            strings::capitalized_sentence(cost_serializer::serialize_cost(cost)),
        )
        .to_string(),
    }
}
