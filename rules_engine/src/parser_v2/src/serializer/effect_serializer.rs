use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;

use super::{
    condition_serializer, cost_serializer, predicate_serializer, serializer_utils,
    static_ability_serializer, text_formatting, trigger_serializer,
};

pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            static_ability_serializer::serialize_standard_static_ability(ability)
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                format!(
                    "until end of turn, {} {}",
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    serializer_utils::capitalize_first_letter(&serialize_effect(&trigger.effect))
                )
            } else {
                format!(
                    "until end of turn, {}{}",
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
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
            text_formatting::card_predicate_base_text(predicate).without_article()
        ),
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => format!(
            "discard a chosen {} from the opponent's hand. They draw {{cards}}.",
            text_formatting::card_predicate_base_text(predicate).without_article()
        ),
        StandardEffect::GainEnergy { .. } => "gain {e}.".to_string(),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => {
                "gain {energy-symbol} equal to that character's cost.".to_string()
            }
            Predicate::This => "gain {energy-symbol} equal to this character's cost.".to_string(),
            _ => format!(
                "gain {{energy-symbol}} equal to {}'s cost.",
                predicate_serializer::serialize_predicate(target)
            ),
        },
        StandardEffect::GainEnergyForEach { for_each, .. } => {
            format!(
                "gain {{e}} for each {}.",
                predicate_serializer::serialize_for_each_predicate(for_each)
            )
        }
        StandardEffect::GainPoints { .. } => "gain {points}.".to_string(),
        StandardEffect::GainPointsForEach { for_count, .. } => {
            format!("gain {{points}} for each {}.", serialize_for_count_expression(for_count))
        }
        StandardEffect::LosePoints { .. } => "you lose {points}.".to_string(),
        StandardEffect::EnemyGainsPoints { .. } => "the opponent gains {points}.".to_string(),
        StandardEffect::Foresee { .. } => "{Foresee}.".to_string(),
        StandardEffect::Kindle { .. } => "{Kindle}.".to_string(),
        StandardEffect::GainsReclaimUntilEndOfTurn { target, cost } => match (target, cost) {
            (Predicate::It, None) => "it gains {reclaim} equal to its cost this turn.".to_string(),
            (_, Some(_)) => {
                format!(
                    "{} gains {{reclaim-for-cost}} this turn.",
                    predicate_serializer::serialize_predicate(target)
                )
            }
            (_, None) => format!(
                "{} gains {{reclaim}} this turn.",
                predicate_serializer::serialize_predicate(target)
            ),
        },
        StandardEffect::GainsSpark { target, .. } => {
            format!("{} gains +{{s}} spark.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::EachMatchingGainsSpark { each, .. } => {
            format!("have each {} gain +{{s}} spark.", serialize_allied_card_predicate(each))
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            format!(
                "each {} gains spark equal to the number of {}.",
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
                    predicate_serializer::serialize_predicate(target),
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
                    predicate_serializer::serialize_card_predicate(matching)
                )
            } else {
                format!(
                    "put {{up-to-n-cards}} {} from your void on top of your deck.",
                    predicate_serializer::serialize_card_predicate_plural(matching)
                )
            }
        }
        StandardEffect::Counterspell { target } => {
            format!("{{Prevent}} a played {}.", predicate_serializer::predicate_base_text(target))
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            format!(
                "{{Prevent}} a played {} unless the opponent pays {}.",
                predicate_serializer::predicate_base_text(target),
                cost_serializer::serialize_cost(cost)
            )
        }
        StandardEffect::GainControl { target } => {
            format!("gain control of {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DissolveCharacter { target } => {
            format!("{{Dissolve}} {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DissolveCharactersCount { target, count } => match count {
            CollectionExpression::All => {
                format!(
                    "{{Dissolve}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "{{Dissolve}} {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::UpTo(n) => {
                format!(
                    "{{Dissolve}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::AnyNumberOf => {
                format!(
                    "{{Dissolve}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            _ => format!("{{Dissolve}} {}.", predicate_serializer::serialize_predicate(target)),
        },
        StandardEffect::BanishCharacter { target } => {
            format!("{{Banish}} {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                format!(
                    "{{Banish}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::All => {
                format!(
                    "{{Banish}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "{{Banish}} {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            CollectionExpression::UpTo(n) => {
                format!(
                    "{{Banish}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            _ => format!("{{Banish}} {}.", predicate_serializer::serialize_predicate(target)),
        },
        StandardEffect::BanishCardsFromEnemyVoid { .. } => {
            "{Banish} {cards} from the opponent's void.".to_string()
        }
        StandardEffect::BanishEnemyVoid => "{Banish} the opponent's void.".to_string(),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            format!(
                "{{Banish}} {} until {} leaves play.",
                predicate_serializer::serialize_predicate(target),
                predicate_serializer::predicate_base_text(until_leaves)
            )
        }
        StandardEffect::BanishUntilNextMain { target } => {
            format!(
                "{{Banish}} {} until your next main phase.",
                predicate_serializer::serialize_predicate(target)
            )
        }
        StandardEffect::Discover { predicate } => {
            format!("{{Discover}} {}.", predicate_serializer::serialize_card_predicate(predicate))
        }
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            format!(
                "{{Discover}} {} and {{materialize}} it.",
                predicate_serializer::serialize_card_predicate(predicate)
            )
        }
        StandardEffect::MaterializeCharacter { target } => {
            format!("{{Materialize}} {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            format!(
                "{{Materialize}} {} at end of turn.",
                predicate_serializer::serialize_predicate(target)
            )
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    format!(
                        "{{Materialize}} a copy of {}.",
                        predicate_serializer::serialize_predicate(target)
                    )
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => {
                    format!(
                        "{{Materialize}} {} copies of {}.",
                        n,
                        predicate_serializer::serialize_predicate(target)
                    )
                }
                (_, QuantityExpression::Matching(predicate)) => {
                    format!(
                        "{{Materialize}} a number of copies of {} equal to the number of {}.",
                        predicate_serializer::serialize_predicate(target),
                        predicate_serializer::serialize_predicate_plural(predicate)
                    )
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    format!(
                        "{{Materialize}} a number of copies of {} equal to the amount of {{energy-symbol}} spent.",
                        predicate_serializer::serialize_predicate(target)
                    )
                }
                (_, quantity_expr) => {
                    format!(
                        "{{Materialize}} a number of copies of {} equal to the number of {}.",
                        predicate_serializer::serialize_predicate(target),
                        serialize_for_count_expression(quantity_expr)
                    )
                }
            }
        }
        StandardEffect::MaterializeFigments { count, .. } => {
            if *count == 1 {
                "{Materialize} {a-figment}.".to_string()
            } else {
                "{Materialize} {n-figments}.".to_string()
            }
        }
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, .. } => {
            let figment_text = if *count == 1 { "{a-figment}" } else { "{n-figments}" };
            match quantity {
                QuantityExpression::PlayedThisTurn(_) => {
                    format!(
                        "{{Materialize}} {} for each card you have played this turn.",
                        figment_text
                    )
                }
                QuantityExpression::Matching(predicate) => {
                    format!(
                        "{{Materialize}} {} for each {}.",
                        figment_text,
                        predicate_serializer::serialize_for_each_predicate(predicate)
                    )
                }
                _ => format!(
                    "{{Materialize}} {} for each {}.",
                    figment_text,
                    serialize_for_count_expression(quantity)
                ),
            }
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                "return an enemy or ally to hand.".to_string()
            }
            Predicate::Another(CardPredicate::Character) => "return an ally to hand.".to_string(),
            Predicate::This => "return this character to your hand.".to_string(),
            _ => format!("return {} to hand.", predicate_serializer::serialize_predicate(target)),
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            format!(
                "return {} from your void to your hand.",
                predicate_serializer::serialize_predicate(target)
            )
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => {
            "return {up-to-n-events} from your void to your hand.".to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            format!("{{Reclaim}} {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            format!(
                "{{Reclaim}} a random {}.",
                predicate_serializer::serialize_card_predicate(predicate)
            )
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            format!(
                "put {} on top of the opponent's deck.",
                predicate_serializer::serialize_predicate(target)
            )
        }
        StandardEffect::EachPlayerDiscardCards { .. } => {
            "each player discards {discards}.".to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            format!(
                "each player abandons {}.",
                predicate_serializer::serialize_card_predicate(matching)
            )
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { .. } => {
            "each player shuffles their hand and void into their deck and then draws {cards}."
                .to_string()
        }
        StandardEffect::CardsInVoidGainReclaimThisTurn { count, predicate } => {
            serialize_cards_in_void_gain_reclaim_this_turn(count, predicate)
        }
        StandardEffect::MaterializeCollection { target, count } => match (target, count) {
            (Predicate::Them, CollectionExpression::All) => "{Materialize} them.".to_string(),
            (_, CollectionExpression::All) => {
                format!(
                    "{{Materialize}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            (_, CollectionExpression::AnyNumberOf) => {
                format!(
                    "{{Materialize}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            (_, CollectionExpression::UpTo(n)) => {
                format!(
                    "{{Materialize}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target)
                )
            }
            _ => format!("{{Materialize}} {}.", predicate_serializer::serialize_predicate(target)),
        },
        StandardEffect::MaterializeRandomFromDeck { predicate, .. } => {
            format!(
                "{{Materialize}} {{n-random-characters}} {} from your deck.",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        StandardEffect::MultiplyYourEnergy { .. } => {
            "{MultiplyBy} the amount of {energy-symbol} you have.".to_string()
        }
        StandardEffect::CopyNextPlayed { matching, .. } => {
            format!(
                "copy the next {} you play {{this-turn-times}}.",
                predicate_serializer::predicate_base_text(matching)
            )
        }
        StandardEffect::Copy { target } => {
            format!("copy {}.", predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => format!(
            "disable the activated abilities of {} while this character is in play.",
            predicate_serializer::serialize_predicate(target)
        ),
        StandardEffect::DrawMatchingCard { predicate } => {
            format!(
                "draw {} from your deck.",
                predicate_serializer::serialize_card_predicate(predicate)
            )
        }
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => format!(
                "trigger the {{Judgment}} ability of each {}.",
                predicate_serializer::predicate_base_text(matching)
            ),
            CollectionExpression::Exactly(1) => format!(
                "trigger the {{Judgment}} ability of {}.",
                predicate_serializer::serialize_predicate(matching)
            ),
            CollectionExpression::Exactly(n) => format!(
                "trigger the {{Judgment}} ability of {} {}.",
                n,
                predicate_serializer::serialize_predicate_plural(matching)
            ),
            _ => format!(
                "trigger the {{Judgment}} ability of {}.",
                predicate_serializer::serialize_predicate(matching)
            ),
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            "at the end of this turn, trigger an additional {JudgmentPhaseName} phase.".to_string()
        }
        StandardEffect::TakeExtraTurn => "take an extra turn after this one.".to_string(),
        StandardEffect::YouWinTheGame => "you win the game.".to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            format!(
                "abandon {} and gain {{energy-symbol}} for each point of spark that character had.",
                predicate_serializer::serialize_predicate(target)
            )
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            format!("abandon {} at end of turn.", predicate_serializer::serialize_predicate(target))
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
                result.push_str(&condition_serializer::serialize_condition(condition));
                result.push(' ');
            }
            if options.optional {
                result.push_str("you may ");
            }
            if let Some(trigger_cost) = &options.trigger_cost {
                result.push_str(&format!(
                    "{} to ",
                    cost_serializer::serialize_trigger_cost(trigger_cost)
                ));
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
                        result.push_str(&condition_serializer::serialize_condition(condition));
                        result.push(' ');
                    }
                }
                result.push_str("you may ");
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    result.push_str(&format!(
                        "{} to ",
                        cost_serializer::serialize_trigger_cost(trigger_cost)
                    ));
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
                        result.push_str(&condition_serializer::serialize_condition(condition));
                        result.push(' ');
                    }
                }
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    result.push_str(&format!(
                        "{} to ",
                        cost_serializer::serialize_trigger_cost(trigger_cost)
                    ));
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
                        result.push_str(&condition_serializer::serialize_condition(condition));
                        result.push(' ');
                    }
                }
                result.push_str(&format!("you may {}.", effect_strings.join(", then ")));
                result
            } else {
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(condition));
                        result.push(' ');
                    }
                }
                let effect_str = effects
                    .iter()
                    .map(|e| {
                        serializer_utils::capitalize_first_letter(&serialize_standard_effect(
                            &e.effect,
                        ))
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push_str(&effect_str);
                result
            }
        }
        Effect::ListWithOptions(list_with_options) => {
            let mut result = String::new();
            if let Some(condition) = &list_with_options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition));
                result.push(' ');
            }
            if let Some(trigger_cost) = &list_with_options.trigger_cost {
                result.push_str(&format!(
                    "{} to ",
                    cost_serializer::serialize_trigger_cost(trigger_cost)
                ));
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
                        effect_str.push_str(&format!(
                            "{} to ",
                            cost_serializer::serialize_trigger_cost(trigger_cost)
                        ));
                    }
                    if let Some(condition) = &e.condition {
                        effect_str.push_str(&condition_serializer::serialize_condition(condition));
                        effect_str.push(' ');
                    }
                    effect_str.push_str(serialize_standard_effect(&e.effect).trim_end_matches('.'));
                    effect_str
                })
                .collect();
            result.push_str(&format!("{}.", effect_strings.join(", then ")));
            result
        }
        Effect::Modal(choices) => {
            let mut result = "{ChooseOne}".to_string();
            for (index, choice) in choices.iter().enumerate() {
                result.push('\n');
                result.push_str("{bullet} ");
                let cost_var = if index == 0 { "{mode1-cost}" } else { "{mode2-cost}" };
                result.push_str(&format!(
                    "{}: {}",
                    cost_var,
                    serializer_utils::capitalize_first_letter(&serialize_effect(&choice.effect))
                ));
            }
            result
        }
    }
}

fn serialize_for_count_expression(quantity_expression: &QuantityExpression) -> String {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => {
            predicate_serializer::serialize_for_each_predicate(predicate)
        }
        QuantityExpression::PlayedThisTurn(predicate) => format!(
            "{} you have played this turn",
            text_formatting::card_predicate_base_text(predicate).without_article()
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
        _ => format!(
            "allied {}",
            text_formatting::card_predicate_base_text(card_predicate).without_article()
        ),
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
                text_formatting::card_predicate_base_text(predicate).capitalized_with_article()
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
