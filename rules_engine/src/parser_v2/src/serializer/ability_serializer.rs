use ability_data::ability::Ability;
use ability_data::named_ability::NamedAbility;
use ability_data::trigger_event::TriggerEvent;

use crate::parser_v2::serializer::{
    cost_serializer, effect_serializer, serializer_utils, static_ability_serializer,
    trigger_serializer,
};
use crate::variables::parser_bindings::VariableBindings;
/// Result of serializing an ability, containing both the text and variable
/// bindings.
pub struct SerializedAbility {
    pub text: String,
    pub variables: VariableBindings,
}
/// Serializes an ability into its rules text and variable bindings.
pub fn serialize_ability(ability: &Ability) -> SerializedAbility {
    let mut variables = VariableBindings::new();
    let text = match ability {
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
            let trigger =
                trigger_serializer::serialize_trigger_event(&triggered.trigger, &mut variables);
            let capitalized_trigger = serializer_utils::capitalize_first_letter(&trigger);
            result.push_str(if has_prefix { &trigger } else { &capitalized_trigger });
            let is_keyword_trigger = matches!(triggered.trigger, TriggerEvent::Keywords(_));
            if is_keyword_trigger {
                result.push(' ');
                result.push_str(&serializer_utils::capitalize_first_letter(
                    &effect_serializer::serialize_effect(&triggered.effect, &mut variables),
                ));
            } else {
                result.push_str(&effect_serializer::serialize_effect(
                    &triggered.effect,
                    &mut variables,
                ));
            }
            result
        }
        Ability::Event(event) => serializer_utils::capitalize_first_letter(
            &effect_serializer::serialize_effect(&event.effect, &mut variables),
        ),
        Ability::Activated(activated) => {
            let mut result = String::new();
            let is_fast = activated.options.as_ref().is_some_and(|options| options.is_fast);
            let has_once_per_turn = activated
                .options
                .as_ref()
                .is_some_and(|options| !options.is_fast && !options.is_multi);
            if is_fast {
                result.push_str("{Fast} -- ");
            }
            let costs = activated
                .costs
                .iter()
                .map(|cost| {
                    serializer_utils::capitalize_first_letter(&cost_serializer::serialize_cost(
                        cost,
                        &mut variables,
                    ))
                })
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&costs);
            if has_once_per_turn {
                result.push_str(", once per turn");
            }
            result.push_str(": ");
            result.push_str(&serializer_utils::capitalize_first_letter(
                &effect_serializer::serialize_effect(&activated.effect, &mut variables),
            ));
            result
        }
        Ability::Named(named) => serialize_named_ability(named, &mut variables),
        Ability::Static(static_ability) => serializer_utils::capitalize_first_letter(
            &static_ability_serializer::serialize_static_ability(static_ability, &mut variables),
        ),
    };
    SerializedAbility { text, variables }
}
fn serialize_named_ability(named: &NamedAbility, variables: &mut VariableBindings) -> String {
    match named {
        NamedAbility::Reclaim(_) => "{ReclaimForCost}".to_string(),
        NamedAbility::ReclaimForCost(cost) => {
            format!(
                "{{Reclaim}} -- {}",
                serializer_utils::capitalize_first_letter(&cost_serializer::serialize_cost(
                    cost, variables
                ))
            )
        }
    }
}
