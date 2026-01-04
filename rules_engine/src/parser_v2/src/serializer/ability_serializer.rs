use ability_data::ability::Ability;
use ability_data::named_ability::NamedAbility;
use ability_data::trigger_event::TriggerEvent;

use super::cost_serializer::serialize_cost;
use super::effect_serializer::serialize_effect;
use super::serializer_utils::capitalize_first_letter;
use super::static_ability_serializer::serialize_static_ability;
use super::trigger_serializer::serialize_trigger_event;

/// Serializes an ability into its rules text as a string.
pub fn serialize_ability(ability: &Ability) -> String {
    match ability {
        Ability::Triggered(triggered) => {
            let mut result = String::new();
            let has_once_per_turn =
                triggered.options.as_ref().map(|o| o.once_per_turn).unwrap_or(false);
            let has_until_end_of_turn =
                triggered.options.as_ref().map(|o| o.until_end_of_turn).unwrap_or(false);
            let has_prefix = has_once_per_turn || has_until_end_of_turn;
            if has_until_end_of_turn {
                result.push_str("Until end of turn, ");
            }
            if has_once_per_turn {
                result.push_str("Once per turn, ");
            }
            let trigger = serialize_trigger_event(&triggered.trigger);
            let capitalized_trigger = capitalize_first_letter(&trigger);
            result.push_str(if has_prefix { &trigger } else { &capitalized_trigger });
            let is_keyword_trigger = matches!(triggered.trigger, TriggerEvent::Keywords(_));
            if is_keyword_trigger {
                result.push(' ');
                result.push_str(&capitalize_first_letter(&serialize_effect(&triggered.effect)));
            } else {
                result.push_str(&serialize_effect(&triggered.effect));
            }
            result
        }
        Ability::Event(event) => capitalize_first_letter(&serialize_effect(&event.effect)),
        Ability::Activated(activated) => {
            let mut result = String::new();
            let has_once_per_turn = activated
                .options
                .as_ref()
                .is_some_and(|options| !options.is_fast && !options.is_multi);
            let costs = activated
                .costs
                .iter()
                .map(|cost| capitalize_first_letter(&serialize_cost(cost)))
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&costs);
            if has_once_per_turn {
                result.push_str(", once per turn");
            }
            result.push_str(": ");
            result.push_str(&capitalize_first_letter(&serialize_effect(&activated.effect)));
            result
        }
        Ability::Named(named) => serialize_named_ability(named),
        Ability::Static(static_ability) => {
            capitalize_first_letter(&serialize_static_ability(static_ability))
        }
    }
}

fn serialize_named_ability(named: &NamedAbility) -> String {
    match named {
        NamedAbility::Reclaim(_) => "{ReclaimForCost}".to_string(),
        NamedAbility::ReclaimForCost(cost) => {
            format!("{{Reclaim}} -- {}", capitalize_first_letter(&serialize_cost(cost)))
        }
    }
}
