use ability_data::collection_expression::CollectionExpression;
use ability_data::condition::Condition;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;

use super::cost_serializer::{serialize_cost, serialize_trigger_cost};
use super::predicate_serializer::{
    serialize_card_predicate, serialize_card_predicate_without_article,
    serialize_for_each_predicate, serialize_predicate, serialize_predicate_plural,
    serialize_predicate_without_article,
};
use super::serializer_utils::capitalize_first_letter;
use super::static_ability_serializer::serialize_standard_static_ability;
use super::trigger_serializer::serialize_trigger_event;

pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            serialize_standard_static_ability(ability)
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                format!(
                    "until end of turn, {} {}",
                    serialize_trigger_event(&trigger.trigger),
                    capitalize_first_letter(&serialize_effect(&trigger.effect))
                )
            } else {
                format!(
                    "until end of turn, {}{}",
                    serialize_trigger_event(&trigger.trigger),
                    serialize_effect(&trigger.effect)
                )
            }
        }
        StandardEffect::DrawCards { .. } => "draw {cards}.".to_string(),
        StandardEffect::DrawCardsForEach { for_each, .. } => {
            format!("draw {{cards}} for each {}.", serialize_for_count_expression(for_each))
        }
        StandardEffect::DiscardCards { .. } => "discard {discards}.".to_string(),
        StandardEffect::DiscardCardFromEnemyHand { predicate } => format!(
            "discard a chosen {} from the opponent's hand.",
            serialize_card_predicate_without_article(predicate)
        ),
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => format!(
            "discard a chosen {} from the opponent's hand. They draw {{cards}}.",
            serialize_card_predicate_without_article(predicate)
        ),
        StandardEffect::GainEnergy { .. } => "gain {e}.".to_string(),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It => "gain {e} equal to that character's cost.".to_string(),
            _ => unimplemented!(
                "Serialization not yet implemented for this GainEnergyEqualTo target"
            ),
        },
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
        StandardEffect::EachMatchingGainsSpark { each, .. } => {
            format!("have each {} gain +{{s}} spark.", serialize_allied_card_predicate(each))
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            format!(
                "each {} gains {{s}} equal to the number of {}.",
                serialize_allied_card_predicate(each),
                serialize_allied_card_predicate(for_each)
            )
        }
        StandardEffect::GainsSparkForQuantity { target, for_quantity, .. } => {
            if matches!(target, Predicate::This) {
                format!(
                    "gain +{{s}} spark for each {}.",
                    serialize_for_count_expression(for_quantity)
                )
            } else {
                format!(
                    "{} gains +{{s}} spark for each {}.",
                    serialize_predicate(target),
                    serialize_for_count_expression(for_quantity)
                )
            }
        }
        StandardEffect::SparkBecomes { matching, .. } => {
            format!(
                "the spark of each {} becomes {{s}}.",
                serialize_allied_card_predicate(matching)
            )
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { .. } => {
            "put the {top-n-cards} of your deck into your void.".to_string()
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                format!(
                    "put {} from your void on top of your deck.",
                    serialize_card_predicate(matching)
                )
            } else {
                unimplemented!("Serialization not yet implemented for this put-on-top count")
            }
        }
        StandardEffect::Counterspell { target } => {
            format!("{{Prevent}} a played {}.", serialize_predicate_without_article(target))
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            format!(
                "{{Prevent}} a played {} unless the opponent pays {}.",
                serialize_predicate_without_article(target),
                serialize_cost(cost)
            )
        }
        StandardEffect::GainControl { target } => {
            format!("gain control of {}.", serialize_predicate(target))
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
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                format!("{{Banish}} any number of {}.", serialize_predicate_plural(target))
            }
            _ => unimplemented!("Serialization not yet implemented for this banish collection"),
        },
        StandardEffect::BanishCardsFromEnemyVoid { .. } => {
            "{Banish} {cards} from the opponent's void.".to_string()
        }
        StandardEffect::BanishEnemyVoid => "{Banish} the opponent's void.".to_string(),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            format!(
                "{{Banish}} {} until {} leaves play.",
                serialize_predicate(target),
                serialize_predicate_without_article(until_leaves)
            )
        }
        StandardEffect::BanishUntilNextMain { target } => {
            format!("{{Banish}} {} until your next main phase.", serialize_predicate(target))
        }
        StandardEffect::Discover { predicate } => {
            format!("{{Discover}} {}.", serialize_card_predicate(predicate))
        }
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            format!("{{Discover}} {} and {{materialize}} it.", serialize_card_predicate(predicate))
        }
        StandardEffect::MaterializeCharacter { target } => {
            format!("{{Materialize}} {}.", serialize_predicate(target))
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            format!("{{Materialize}} {} at end of turn.", serialize_predicate(target))
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            if *count == 1 && matches!(quantity, QuantityExpression::Matching(_)) {
                format!("{{Materialize}} a copy of {}.", serialize_predicate(target))
            } else {
                unimplemented!("Serialization not yet implemented for this materialize copy effect")
            }
        }
        StandardEffect::MaterializeFigments { .. } => "{Materialize} {n-figments}.".to_string(),
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, .. } => {
            if *count == 1 && matches!(quantity, QuantityExpression::PlayedThisTurn(_)) {
                "{Materialize} {a-figment} for each card you have played this turn.".to_string()
            } else {
                unimplemented!(
                    "Serialization not yet implemented for this materialize figments quantity"
                )
            }
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                "return an enemy or ally to hand.".to_string()
            }
            Predicate::Another(CardPredicate::Character) => "return an ally to hand.".to_string(),
            Predicate::This => "return this character to your hand.".to_string(),
            _ => format!("return {} to hand.", serialize_predicate(target)),
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            format!("return {} from your void to your hand.", serialize_predicate(target))
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => {
            "return {up-to-n-events} from your void to your hand.".to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            format!("{{Reclaim}} {}.", serialize_predicate(target))
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            format!("put {} on top of the opponent's deck.", serialize_predicate(target))
        }
        StandardEffect::EachPlayerDiscardCards { .. } => {
            "each player discards {discards}.".to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            format!("each player abandons {}.", serialize_card_predicate(matching))
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { .. } => {
            "each player shuffles their hand and void into their deck and then draws {cards}."
                .to_string()
        }
        StandardEffect::CardsInVoidGainReclaimThisTurn { count, predicate } => {
            serialize_cards_in_void_gain_reclaim_this_turn(count, predicate)
        }
        StandardEffect::MaterializeCollection { target, count } => {
            if matches!(target, Predicate::Them) && matches!(count, CollectionExpression::All) {
                "{Materialize} them.".to_string()
            } else {
                unimplemented!(
                    "Serialization not yet implemented for this materialize collection pattern"
                )
            }
        }
        StandardEffect::MaterializeRandomFromDeck { predicate, .. } => {
            format!(
                "{{Materialize}} {{n-random-characters}} {} from your deck.",
                serialize_card_predicate_without_article(predicate)
            )
        }
        StandardEffect::MultiplyYourEnergy { .. } => {
            "{MultiplyBy} the amount of {energy-symbol} you have.".to_string()
        }
        StandardEffect::CopyNextPlayed { matching, .. } => {
            format!(
                "copy the next {} you play {{this-turn-times}}.",
                serialize_predicate_without_article(matching)
            )
        }
        StandardEffect::Copy { target } => {
            format!("copy {}.", serialize_predicate(target))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => format!(
            "disable the activated abilities of {} while this character is in play.",
            serialize_predicate(target)
        ),
        StandardEffect::DrawMatchingCard { predicate } => {
            format!("draw {} from your deck.", serialize_card_predicate(predicate))
        }
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            "at the end of this turn, trigger an additional {JudgmentPhaseName} phase.".to_string()
        }
        _ => unimplemented!("Serialization not yet implemented for this effect type"),
    }
}

pub fn serialize_effect(effect: &Effect) -> String {
    match effect {
        Effect::Effect(standard_effect) => serialize_standard_effect(standard_effect),
        Effect::WithOptions(options) => {
            let mut result = String::new();
            if let Some(condition) = &options.condition {
                result.push_str(&serialize_condition(condition));
                result.push(' ');
            }
            if options.optional {
                result.push_str("you may ");
            }
            if let Some(trigger_cost) = &options.trigger_cost {
                result.push_str(&format!("{} to ", serialize_trigger_cost(trigger_cost)));
            }
            result.push_str(&serialize_standard_effect(&options.effect));
            result
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let has_condition = effects.first().and_then(|e| e.condition.as_ref()).is_some();
            let all_have_trigger_cost = effects.iter().all(|e| e.trigger_cost.is_some());
            if all_optional && all_have_trigger_cost && !effects.is_empty() {
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
                result.push_str("you may ");
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    result.push_str(&format!("{} to ", serialize_trigger_cost(trigger_cost)));
                }
                result.push_str(&format!("{}.", effect_strings.join(" and ")));
                result
            } else if !all_optional && all_have_trigger_cost && !effects.is_empty() {
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
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    result.push_str(&format!("{} to ", serialize_trigger_cost(trigger_cost)));
                }
                result.push_str(&format!("{}.", effect_strings.join(" and ")));
                result
            } else if all_optional && !effects.is_empty() {
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
        Effect::ListWithOptions(list_with_options) => {
            let mut result = String::new();
            if let Some(condition) = &list_with_options.condition {
                result.push_str(&serialize_condition(condition));
                result.push(' ');
            }
            if let Some(trigger_cost) = &list_with_options.trigger_cost {
                result.push_str(&format!("{} to ", serialize_trigger_cost(trigger_cost)));
            }
            let effect_strings: Vec<String> = list_with_options
                .effects
                .iter()
                .map(|e| {
                    let mut effect_str = String::new();
                    if e.optional {
                        effect_str.push_str("you may ");
                    }
                    if let Some(trigger_cost) = &e.trigger_cost {
                        effect_str
                            .push_str(&format!("{} to ", serialize_trigger_cost(trigger_cost)));
                    }
                    if let Some(condition) = &e.condition {
                        effect_str.push_str(&serialize_condition(condition));
                        effect_str.push(' ');
                    }
                    effect_str.push_str(serialize_standard_effect(&e.effect).trim_end_matches('.'));
                    effect_str
                })
                .collect();
            result.push_str(&format!("{}.", effect_strings.join(", then ")));
            result
        }
        Effect::Modal(_) => {
            unimplemented!("Serialization not yet implemented for modal effects")
        }
    }
}

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::AlliesThatShareACharacterType { .. } => {
            "with {count-allies} that share a character type,".to_string()
        }
        Condition::PredicateCount { count, predicate } => {
            format!("with {},", serialize_predicate_count(*count, predicate))
        }
        Condition::DissolvedThisTurn { .. } => "if a character dissolved this turn".to_string(),
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

fn serialize_for_count_expression(quantity_expression: &QuantityExpression) -> String {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => serialize_for_each_predicate(predicate),
        QuantityExpression::PlayedThisTurn(predicate) => format!(
            "{} you have played this turn",
            serialize_card_predicate_without_article(predicate)
        ),
        QuantityExpression::AbandonedThisTurn(CardPredicate::Character) => {
            "ally abandoned this turn".to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::Character) => {
            "ally abandoned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character) => {
            "ally returned".to_string()
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => "{energy-symbol} spent".to_string(),
        _ => {
            unimplemented!("Serialization not yet implemented for this quantity expression")
        }
    }
}

fn serialize_allied_card_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::CharacterType(_) => "allied {subtype}".to_string(),
        _ => format!("allied {}", serialize_card_predicate_without_article(card_predicate)),
    }
}

fn serialize_cards_in_void_gain_reclaim_this_turn(
    count: &CollectionExpression,
    predicate: &CardPredicate,
) -> String {
    match count {
        CollectionExpression::Exactly(1) => {
            format!(
                "{} in your void gains {{reclaim}} equal to its cost.",
                serialize_card_predicate(predicate)
            )
        }
        CollectionExpression::All => {
            "all cards currently in your void gain {reclaim} equal to their cost this turn."
                .to_string()
        }
        _ => unimplemented!(
            "Serialization not yet implemented for this collection expression in cards in void gain reclaim"
        ),
    }
}
