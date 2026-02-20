use std::collections::BTreeMap;

use ability_data::ability::Ability;
use ability_data::activated_ability::ActivatedAbility;
use ability_data::cost::Cost;
use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use ability_data::named_ability::NamedAbility;
use ability_data::trigger_event::TriggerEvent;
use ability_data::triggered_ability::TriggeredAbility;
use rlf::Phrase;
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
    SerializedAbility { text }
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
    SerializedAbility { text }
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
                result.insert(ModelEffectChoiceIndex(current_index), SerializedAbility { text });
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

/// Builds the combined prefix phrase for trigger modifiers.
fn build_trigger_prefix(has_until_end_of_turn: bool, has_once_per_turn: bool) -> Phrase {
    let prefix = Phrase::empty();
    let prefix = if has_until_end_of_turn { strings::until_end_of_turn_prefix() } else { prefix };
    if has_once_per_turn {
        prefix.map_text(|t| format!("{t}{}", strings::once_per_turn_prefix()))
    } else {
        prefix
    }
}

/// Assembles an activated ability using phrase-based composition.
fn serialize_activated(activated: &ActivatedAbility) -> String {
    let is_fast = activated.options.as_ref().is_some_and(|options| options.is_fast);
    let has_once_per_turn = activated.options.as_ref().is_some_and(|options| !options.is_multi);
    let costs = join_activated_costs(&activated.costs);
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

/// Joins activated ability costs with correct separators and
/// capitalization.
///
/// Energy costs are separated from non-energy costs by the activated
/// cost separator. Non-energy costs use a distinct separator between
/// items, with a potentially different final separator before the last
/// item (e.g. ", " between items but " и " before the last in
/// Russian). The first cost is always capitalized; subsequent
/// non-energy costs are wrapped with `activated_subsequent_cost`.
fn join_activated_costs(costs: &[Cost]) -> String {
    let mut energy_parts = Vec::new();
    let mut non_energy_parts = Vec::new();
    for cost in costs {
        if matches!(cost, Cost::Energy(_)) {
            energy_parts.push(cost_serializer::serialize_cost(cost).to_string());
        } else {
            non_energy_parts.push(cost_serializer::serialize_cost(cost));
        }
    }

    let cost_separator = strings::activated_cost_separator().to_string();

    if energy_parts.is_empty() {
        // No energy costs: capitalize first non-energy cost, apply
        // subsequent formatting to the rest.
        let non_energy_strings: Vec<String> = non_energy_parts
            .into_iter()
            .enumerate()
            .map(|(i, phrase)| {
                if i == 0 {
                    strings::capitalized_sentence(phrase).to_string()
                } else {
                    strings::activated_subsequent_cost(phrase).to_string()
                }
            })
            .collect();
        join_with_final_separator(&non_energy_strings)
    } else {
        // Energy costs present: join energy parts, then append
        // non-energy costs with subsequent formatting using the standard
        // separator between groups and the non-energy separator within.
        let energy_str = energy_parts.join(&cost_separator);
        if non_energy_parts.is_empty() {
            energy_str
        } else {
            let non_energy_strings: Vec<String> = non_energy_parts
                .into_iter()
                .map(|p| strings::activated_subsequent_cost(p).to_string())
                .collect();
            format!(
                "{energy_str}{cost_separator}{}",
                join_with_final_separator(&non_energy_strings)
            )
        }
    }
}

/// Joins strings using the non-energy cost separator, with a
/// potentially different separator before the final element.
fn join_with_final_separator(items: &[String]) -> String {
    let len = items.len();
    match len {
        0 => String::new(),
        1 => items[0].clone(),
        _ => {
            let separator = strings::activated_non_energy_cost_separator().to_string();
            let final_separator = strings::activated_final_non_energy_cost_separator().to_string();
            let mut result = items[0].clone();
            for (i, item) in items.iter().enumerate().skip(1) {
                if i == len - 1 {
                    result.push_str(&final_separator);
                } else {
                    result.push_str(&separator);
                }
                result.push_str(item);
            }
            result
        }
    }
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
        NamedAbility::ReclaimForCost(cost) => {
            strings::capitalized_sentence(strings::reclaim_with_cost(
                strings::capitalized_sentence(cost_serializer::serialize_cost(cost)),
            ))
            .to_string()
        }
    }
}
