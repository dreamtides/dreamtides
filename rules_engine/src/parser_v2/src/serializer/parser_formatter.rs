use ability_data::ability::Ability;
use ability_data::condition::Condition;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::named_ability::NamedAbility;
use ability_data::predicate::{CardPredicate, Operator, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::static_ability::{StandardStaticAbility, StaticAbility};
use ability_data::trigger_event::{TriggerEvent, TriggerKeyword};
pub fn serialize_ability(ability: &Ability) -> String {
    match ability {
        Ability::Triggered(triggered) => {
            let mut result = String::new();
            let has_once_per_turn =
                triggered.options.as_ref().map(|o| o.once_per_turn).unwrap_or(false);
            if has_once_per_turn {
                result.push_str("Once per turn, ");
            }
            let trigger = serialize_trigger_event(&triggered.trigger);
            let capitalized_trigger = capitalize_first_letter(&trigger);
            result.push_str(if has_once_per_turn { &trigger } else { &capitalized_trigger });
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
            let costs = activated.costs.iter().map(serialize_cost).collect::<Vec<_>>().join(", ");
            result.push_str(&capitalize_first_letter(&costs));
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
pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::DrawCards { .. } => "draw {cards}.".to_string(),
        StandardEffect::DiscardCards { .. } => "discard {discards}.".to_string(),
        StandardEffect::GainEnergy { .. } => "gain {e}.".to_string(),
        StandardEffect::GainEnergyForEach { for_each, .. } => {
            format!("gain {{e}} for each {}.", serialize_for_each_predicate(for_each))
        }
        StandardEffect::GainPoints { .. } => "gain {points}.".to_string(),
        StandardEffect::GainPointsForEach { for_count, .. } => {
            format!("gain {{points}} for each {}.", serialize_for_count_expression(for_count))
        }
        StandardEffect::LosePoints { .. } => "you lose {points}.".to_string(),
        StandardEffect::EnemyGainsPoints { .. } => "the opponent gains {points}.".to_string(),
        StandardEffect::Foresee { .. } => "{Foresee}.".to_string(),
        StandardEffect::Kindle { .. } => "{Kindle}.".to_string(),
        StandardEffect::GainsSpark { target, .. } => {
            format!("{} gains +{{s}} spark.", serialize_predicate(target))
        }
        StandardEffect::Counterspell { target } => {
            format!("{{Prevent}} {}.", serialize_predicate(target))
        }
        StandardEffect::DissolveCharacter { target } => {
            format!("{{Dissolve}} {}.", serialize_predicate(target))
        }
        StandardEffect::DissolveCharactersCount { target, count } => {
            use ability_data::collection_expression::CollectionExpression;
            match count {
                CollectionExpression::All => match target {
                    Predicate::Any(CardPredicate::Character) => {
                        "{Dissolve} all characters.".to_string()
                    }
                    _ => unimplemented!("Unsupported dissolve all characters target"),
                },
                _ => unimplemented!("Unsupported dissolve characters count"),
            }
        }
        StandardEffect::BanishCharacter { target } => {
            format!("{{Banish}} {}.", serialize_predicate(target))
        }
        StandardEffect::Discover { predicate } => {
            format!("{{Discover}} {}.", serialize_card_predicate(predicate))
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                "return an enemy or ally to hand.".to_string()
            }
            Predicate::Another(CardPredicate::Character) => "return an ally to hand.".to_string(),
            _ => format!("return {} to hand.", serialize_predicate(target)),
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            format!("return {} from your void to your hand.", serialize_predicate(target))
        }
        _ => unimplemented!("Serialization not yet implemented for this effect type"),
    }
}
pub fn serialize_trigger_event(trigger: &TriggerEvent) -> String {
    match trigger {
        TriggerEvent::Keywords(keywords) if keywords.len() == 1 => {
            format!("{{{}}}", serialize_keyword(&keywords[0]))
        }
        TriggerEvent::Keywords(keywords) if keywords.len() == 2 => {
            format!("{{{}{}}}", serialize_keyword(&keywords[0]), serialize_keyword(&keywords[1]))
        }
        TriggerEvent::Play(predicate) => {
            format!("when you play {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::PlayFromHand(predicate) => {
            format!("when you play {} from your hand, ", serialize_predicate(predicate))
        }
        TriggerEvent::PlayDuringTurn(predicate, _) => {
            format!("when you play {} in a turn, ", serialize_predicate(predicate))
        }
        TriggerEvent::Discard(predicate) => {
            format!("when you discard {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Materialize(predicate) => {
            format!("when you {{materialize}} {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Dissolved(predicate) => {
            format!("when {} is {{dissolved}}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Banished(predicate) => {
            format!("when {} is {{banished}}, ", serialize_predicate(predicate))
        }
        TriggerEvent::Abandon(predicate) => {
            format!("when you abandon {}, ", serialize_predicate(predicate))
        }
        TriggerEvent::EndOfYourTurn => "at the end of your turn, ".to_string(),
        TriggerEvent::DrawAllCardsInCopyOfDeck => {
            "when you have no cards in your deck, ".to_string()
        }
        TriggerEvent::GainEnergy => "when you gain energy, ".to_string(),
        _ => unimplemented!("Serialization not yet implemented for this trigger type"),
    }
}
fn serialize_static_ability(static_ability: &StaticAbility) -> String {
    match static_ability {
        StaticAbility::StaticAbility(ability) => serialize_standard_static_ability(ability),
        StaticAbility::WithOptions(ability) => {
            if ability.condition.is_none() {
                serialize_standard_static_ability(&ability.ability)
            } else {
                unimplemented!("Serialization not yet implemented for this static ability")
            }
        }
    }
}
fn serialize_standard_static_ability(ability: &StandardStaticAbility) -> String {
    match ability {
        StandardStaticAbility::YourCardsCostIncrease { matching, .. } => {
            format!("{} cost you {{e}} more.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::YourCardsCostReduction { matching, .. } => {
            format!("{} cost you {{e}} less.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::EnemyCardsCostIncrease { matching, .. } => {
            format!("the opponent's {} cost {{e}} more.", serialize_card_predicate_plural(matching))
        }
        StandardStaticAbility::SparkBonusOtherCharacters { matching, .. } => {
            format!("allied {} have +{{s}} spark.", serialize_card_predicate_plural(matching))
        }
        _ => unimplemented!("Serialization not yet implemented for this static ability"),
    }
}
fn serialize_cost(cost: &Cost) -> String {
    match cost {
        Cost::AbandonCharacters(predicate, _) => {
            format!("abandon {}", serialize_predicate(predicate))
        }
        _ => unimplemented!("Serialization not yet implemented for this cost type"),
    }
}
fn serialize_effect(effect: &Effect) -> String {
    match effect {
        Effect::Effect(standard_effect) => serialize_standard_effect(standard_effect),
        Effect::WithOptions(options) => {
            let mut result = String::new();
            if let Some(condition) = &options.condition {
                result.push_str(&serialize_condition(condition));
                result.push(' ');
            }
            if options.optional {
                result.push_str(&format!("you may {}", serialize_standard_effect(&options.effect)));
            } else {
                result.push_str(&serialize_standard_effect(&options.effect));
            }
            result
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let has_condition = effects.first().and_then(|e| e.condition.as_ref()).is_some();
            if all_optional && !effects.is_empty() {
                let effect_strings: Vec<String> = effects
                    .iter()
                    .map(|e| serialize_standard_effect(&e.effect).trim_end_matches('.').to_string())
                    .collect();
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&serialize_condition(condition));
                        result.push(' ');
                    }
                }
                result.push_str(&format!("you may {}.", effect_strings.join(", then ")));
                result
            } else {
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&serialize_condition(condition));
                        result.push(' ');
                    }
                }
                let effect_str = effects
                    .iter()
                    .map(|e| capitalize_first_letter(&serialize_standard_effect(&e.effect)))
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push_str(&effect_str);
                result
            }
        }
        Effect::Modal(_) => {
            unimplemented!("Serialization not yet implemented for modal effects")
        }
    }
}
fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::PredicateCount { count, predicate } => {
            format!("with {},", serialize_predicate_count(*count, predicate))
        }
        _ => unimplemented!("Serialization not yet implemented for this condition type"),
    }
}
fn serialize_predicate_count(_count: u32, predicate: &Predicate) -> String {
    match predicate {
        Predicate::Another(CardPredicate::CharacterType(_)) => "{count-allied-subtype}".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this predicate count type")
        }
    }
}
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
fn serialize_keyword(keyword: &TriggerKeyword) -> String {
    match keyword {
        TriggerKeyword::Judgment => "Judgment".to_string(),
        TriggerKeyword::Materialized => "Materialized".to_string(),
        TriggerKeyword::Dissolved => "Dissolved".to_string(),
    }
}
fn serialize_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::Your(card_predicate) => {
            format!("an {}", serialize_your_predicate(card_predicate))
        }
        Predicate::Another(card_predicate) => {
            format!("an {}", serialize_your_predicate(card_predicate))
        }
        Predicate::Any(card_predicate) => serialize_card_predicate(card_predicate),
        Predicate::Enemy(card_predicate) => {
            format!("an {}", serialize_enemy_predicate(card_predicate))
        }
        _ => unimplemented!("Serialization not yet implemented for this predicate type"),
    }
}
fn serialize_your_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "ally".to_string(),
        CardPredicate::CharacterType(_) => "allied {subtype}".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this your predicate type")
        }
    }
}
fn serialize_enemy_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "enemy".to_string(),
        CardPredicate::CharacterWithSpark(_, operator) => {
            format!("enemy with spark {{s}} {}", serialize_operator(operator))
        }
        CardPredicate::CardWithCost { cost_operator, .. } => {
            format!("enemy with cost {{e}} {}", serialize_operator(cost_operator))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this enemy predicate type")
        }
    }
}
fn serialize_card_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card => "a card".to_string(),
        CardPredicate::Character => "a character".to_string(),
        CardPredicate::Event => "an event".to_string(),
        CardPredicate::CharacterType(_) => "{a-subtype}".to_string(),
        CardPredicate::Fast { target } => {
            format!("a {{fast}} {}", serialize_fast_target(target))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this card predicate type")
        }
    }
}
fn serialize_card_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card => "cards".to_string(),
        CardPredicate::Character => "characters".to_string(),
        CardPredicate::Event => "events".to_string(),
        CardPredicate::CharacterType(_) => "{plural-subtype}".to_string(),
        CardPredicate::Fast { target } => {
            format!("fast {}", serialize_card_predicate_plural(target))
        }
        _ => {
            unimplemented!("Serialization not yet implemented for this card predicate type")
        }
    }
}
fn serialize_fast_target(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Card => "card".to_string(),
        CardPredicate::Character => "character".to_string(),
        CardPredicate::Event => "event".to_string(),
        CardPredicate::CharacterType(_) => "{subtype}".to_string(),
        CardPredicate::CharacterWithSpark(_spark, operator) => {
            format!("character with spark {{s}} {}", serialize_operator(operator))
        }
        CardPredicate::CardWithCost { target, cost_operator, .. } => {
            format!(
                "{} with cost {{e}} {}",
                serialize_fast_target(target),
                serialize_operator(cost_operator)
            )
        }
        _ => unimplemented!("Unsupported fast target"),
    }
}
fn serialize_operator<T>(operator: &Operator<T>) -> String {
    match operator {
        Operator::OrLess => "or less".to_string(),
        Operator::OrMore => "or more".to_string(),
        Operator::Exactly => "exactly".to_string(),
        Operator::LowerBy(_) => "lower".to_string(),
        Operator::HigherBy(_) => "higher".to_string(),
    }
}
fn serialize_named_ability(named: &NamedAbility) -> String {
    match named {
        NamedAbility::Reclaim(_) => "{ReclaimForCost}".to_string(),
    }
}
fn serialize_for_each_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Another(CardPredicate::Character) => "allied character".to_string(),
        Predicate::Another(CardPredicate::CharacterType(_)) => "allied {subtype}".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this for-each predicate")
        }
    }
}
fn serialize_for_count_expression(quantity_expression: &QuantityExpression) -> String {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => serialize_for_each_predicate(predicate),
        _ => {
            unimplemented!("Serialization not yet implemented for this quantity expression")
        }
    }
}
