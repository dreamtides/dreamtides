use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;
use ability_data::variable_value::VariableValue;
use core_data::numerics::Energy;
use strings::strings;

use crate::serializer::{
    condition_serializer, cost_serializer, predicate_serializer, serializer_utils,
    static_ability_serializer, trigger_serializer,
};
use crate::variables::parser_bindings::VariableBindings;

/// Context for effect serialization to determine joining behavior.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AbilityContext {
    /// Triggered ability - use `, then` for mandatory effect lists.
    Triggered,
    /// Event or other ability - use `. ` for mandatory effect lists.
    #[default]
    Event,
}

pub fn serialize_standard_effect(
    effect: &StandardEffect,
    bindings: &mut VariableBindings,
) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            static_ability_serializer::serialize_standard_static_ability(ability, bindings)
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                format!(
                    "until end of turn, {} {}",
                    trigger_serializer::serialize_trigger_event(&trigger.trigger, bindings),
                    serializer_utils::capitalize_first_letter(&serialize_effect(
                        &trigger.effect,
                        bindings
                    ))
                )
            } else {
                format!(
                    "until end of turn, {}{}",
                    trigger_serializer::serialize_trigger_event(&trigger.trigger, bindings),
                    serialize_effect(&trigger.effect, bindings)
                )
            }
        }
        StandardEffect::DrawCards { count } => strings::draw_cards_effect(*count).to_string(),
        StandardEffect::DrawCardsForEach { count, for_each } => {
            let target = serialize_for_count_expression(for_each, bindings);
            format!("{}.", strings::draw_cards_for_each(*count, target))
        }
        StandardEffect::DiscardCards { count } => strings::discard_cards_effect(*count).to_string(),
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            let target =
                predicate_serializer::serialize_card_predicate_without_article(predicate, bindings);
            format!("{}.", strings::discard_chosen_from_enemy_hand(target))
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            bindings.insert("c".to_string(), VariableValue::Integer(1));
            format!(
                "discard a chosen {} from the opponent's hand. They draw {{cards($c)}}.",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        StandardEffect::GainEnergy { gains } => strings::gain_energy_effect(gains.0).to_string(),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => {
                strings::gain_energy_equal_to_that_cost_effect().to_string()
            }
            Predicate::This => strings::gain_energy_equal_to_this_cost_effect().to_string(),
            _ => {
                let target = predicate_serializer::serialize_predicate(target, bindings);
                format!("{}.", strings::gain_energy_equal_to_cost(target))
            }
        },
        StandardEffect::GainEnergyForEach { gains, for_each } => {
            let target = predicate_serializer::serialize_for_each_predicate(for_each, bindings);
            format!("{}.", strings::gain_energy_for_each(gains.0, target))
        }
        StandardEffect::GainPoints { gains } => strings::gain_points_effect(gains.0).to_string(),
        StandardEffect::GainPointsForEach { gain, for_count } => {
            let target = serialize_for_count_expression(for_count, bindings);
            format!("{}.", strings::gain_points_for_each(gain.0, target))
        }
        StandardEffect::LosePoints { loses } => strings::lose_points_effect(loses.0).to_string(),
        StandardEffect::EnemyGainsPoints { count } => {
            strings::opponent_gains_points_effect(*count).to_string()
        }
        StandardEffect::EnemyGainsPointsEqualToItsSpark => {
            strings::opponent_gains_points_equal_spark().to_string()
        }
        StandardEffect::EnemyLosesPoints { count } => {
            strings::opponent_loses_points_effect(*count).to_string()
        }
        StandardEffect::Foresee { count } => strings::foresee_effect(*count).to_string(),
        StandardEffect::Kindle { amount } => strings::kindle_effect(amount.0).to_string(),
        StandardEffect::GainsReclaim { target, count, this_turn, cost } => {
            serialize_gains_reclaim(target, count, *this_turn, cost, bindings)
        }
        StandardEffect::GainsSpark { target, gains } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::gains_spark(target, gains.0))
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            let each_text = serialize_allied_card_predicate(each, bindings);
            format!("{}.", strings::have_each_gain_spark(each_text, gains.0))
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            let each_text = serialize_allied_card_predicate(each, bindings);
            let count_of = serialize_allied_card_predicate_plural(for_each, bindings);
            format!("{}.", strings::each_gains_spark_equal_to(each_text, count_of))
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            let quantity = serialize_for_count_expression(for_quantity, bindings);
            format!("{}.", strings::gains_spark_for_each(target, gains.0, quantity))
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            let each_text = serialize_allied_card_predicate(matching, bindings);
            format!("{}.", strings::spark_of_each_becomes(each_text, spark.0))
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            strings::put_deck_into_void_effect(*count).to_string()
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                format!(
                    "put {} from your void on top of your deck.",
                    predicate_serializer::serialize_card_predicate(matching, bindings)
                )
            } else {
                bindings.insert("c".to_string(), VariableValue::Integer(*count));
                format!(
                    "put up to {{cards($c)}} {} from your void on top of your deck.",
                    predicate_serializer::serialize_card_predicate_plural(matching, bindings)
                )
            }
        }
        StandardEffect::Counterspell { target } => {
            // For "that card" references, don't add "a played" prefix
            if matches!(target, Predicate::That | Predicate::It) {
                strings::prevent_that_card_effect().to_string()
            } else {
                let target = predicate_serializer::predicate_base_text(target, bindings);
                format!("{}.", strings::prevent_played_target(target))
            }
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            let target = predicate_serializer::predicate_base_text(target, bindings);
            let cost = cost_serializer::serialize_cost(cost, bindings);
            format!("{}.", strings::prevent_unless_pays(target, cost))
        }
        StandardEffect::GainControl { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::gain_control_of(target))
        }
        StandardEffect::DissolveCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::dissolve_target(target))
        }
        StandardEffect::DissolveCharactersCount { target, count } => match count {
            CollectionExpression::All => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::dissolve_all(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::dissolve_exactly(*n, target))
            }
            CollectionExpression::UpTo(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::dissolve_up_to(*n, target))
            }
            CollectionExpression::AnyNumberOf => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::dissolve_any_number_of(target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target, bindings);
                format!("{}.", strings::dissolve_single(target))
            }
        },
        StandardEffect::BanishCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::banish_target(target))
        }
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::banish_any_number_of(target))
            }
            CollectionExpression::All => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::banish_all(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::banish_exactly(*n, target))
            }
            CollectionExpression::UpTo(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::banish_up_to(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target, bindings);
                format!("{}.", strings::banish_single(target))
            }
        },
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            strings::banish_cards_from_enemy_void_effect(*count).to_string()
        }
        StandardEffect::BanishEnemyVoid => strings::banish_enemy_void_effect().to_string(),
        StandardEffect::BanishThenMaterialize { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                format!(
                    "{{banish}} {}, then {{materialize}} it.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
            CollectionExpression::AnyNumberOf => {
                format!(
                    "{{banish}} any number of {}, then {{materialize}} them.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::UpTo(n) => {
                bindings.insert("n".to_string(), VariableValue::Integer(*n));
                "{banish} {up_to_n_allies($n)}, then {materialize} {pronoun:$n}.".to_string()
            }
            _ => {
                format!(
                    "{{banish}} {}, then {{materialize}} them.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
        },
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            let until = predicate_serializer::predicate_base_text(until_leaves, bindings);
            format!("{}.", strings::banish_until_leaves(target, until))
        }
        StandardEffect::BanishUntilNextMain { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::banish_until_next_main(target))
        }
        StandardEffect::Discover { predicate } => {
            let target = predicate_serializer::serialize_card_predicate(predicate, bindings);
            format!("{}.", strings::discover_target(target))
        }
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            let target = predicate_serializer::serialize_card_predicate(predicate, bindings);
            format!("{}.", strings::discover_and_materialize(target))
        }
        StandardEffect::MaterializeCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::materialize_target(target))
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::materialize_at_end_of_turn(target))
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    let target = predicate_serializer::serialize_predicate(target, bindings);
                    format!("{}.", strings::materialize_copy_of(target))
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => {
                    let target = predicate_serializer::serialize_predicate(target, bindings);
                    format!("{}.", strings::materialize_n_copies_of(*n, target))
                }
                (_, QuantityExpression::Matching(predicate)) => {
                    let target = predicate_serializer::serialize_predicate(target, bindings);
                    let matching =
                        predicate_serializer::serialize_predicate_plural(predicate, bindings);
                    format!("{}.", strings::materialize_copies_equal_to_matching(target, matching))
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    let target = predicate_serializer::serialize_predicate(target, bindings);
                    format!("{}.", strings::materialize_copies_equal_to_energy(target))
                }
                (_, quantity_expr) => {
                    let target = predicate_serializer::serialize_predicate(target, bindings);
                    let quantity = serialize_for_count_expression(quantity_expr, bindings);
                    format!("{}.", strings::materialize_copies_equal_to_quantity(target, quantity))
                }
            }
        }
        StandardEffect::MaterializeFigments { count, figment } => {
            bindings.insert("g".to_string(), VariableValue::Figment(*figment));
            if *count == 1 {
                "{materialize} {@a figment($g)}.".to_string()
            } else {
                bindings.insert("n".to_string(), VariableValue::Integer(*count));
                "{materialize} {n_figments($n, $g)}.".to_string()
            }
        }
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, figment } => {
            bindings.insert("g".to_string(), VariableValue::Figment(*figment));
            let figment_text = if *count == 1 {
                "{@a figment($g)}".to_string()
            } else {
                bindings.insert("n".to_string(), VariableValue::Integer(*count));
                "{n_figments($n, $g)}".to_string()
            };
            match quantity {
                QuantityExpression::PlayedThisTurn(_) => {
                    format!("{}.", strings::materialize_figments_for_each_played(figment_text))
                }
                QuantityExpression::Matching(predicate) => {
                    let for_each =
                        predicate_serializer::serialize_for_each_predicate(predicate, bindings);
                    format!("{}.", strings::materialize_figments_for_each(figment_text, for_each))
                }
                _ => {
                    let quantity_text = serialize_for_count_expression(quantity, bindings);
                    format!(
                        "{}.",
                        strings::materialize_figments_for_each_quantity(
                            figment_text,
                            quantity_text
                        )
                    )
                }
            }
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                format!("{}.", strings::return_any_character_to_hand())
            }
            Predicate::Another(CardPredicate::Character) => {
                format!("{}.", strings::return_ally_to_hand())
            }
            Predicate::This => format!("{}.", strings::return_this_to_hand()),
            _ => {
                let target = predicate_serializer::serialize_predicate(target, bindings);
                format!("{}.", strings::return_to_hand(target))
            }
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            let target = match target {
                Predicate::YourVoid(card_predicate) => {
                    predicate_serializer::serialize_card_predicate(card_predicate, bindings)
                }
                _ => predicate_serializer::serialize_predicate(target, bindings),
            };
            format!("{}.", strings::return_from_void_to_hand(target))
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => {
            strings::return_up_to_events_from_void_effect(*count).to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::reclaim_target(target))
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            let target =
                predicate_serializer::serialize_card_predicate_without_article(predicate, bindings);
            format!("{}.", strings::reclaim_random(target))
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::put_on_top_of_enemy_deck(target))
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            strings::each_player_discards_effect(*count).to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            let target = predicate_serializer::serialize_card_predicate(matching, bindings);
            format!("{}.", strings::each_player_abandons(target))
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count } => {
            strings::each_player_shuffles_and_draws_effect(*count).to_string()
        }
        StandardEffect::MaterializeCollection { target, count } => match (target, count) {
            (Predicate::Them, CollectionExpression::All) => {
                format!("{}.", strings::materialize_them())
            }
            (_, CollectionExpression::All) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::materialize_all(target))
            }
            (_, CollectionExpression::AnyNumberOf) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::materialize_any_number_of(target))
            }
            (_, CollectionExpression::UpTo(n)) => {
                let target = predicate_serializer::serialize_predicate_plural(target, bindings);
                format!("{}.", strings::materialize_up_to(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target, bindings);
                format!("{}.", strings::materialize_single(target))
            }
        },
        StandardEffect::MaterializeRandomFromDeck { count, predicate } => {
            bindings.insert("n".to_string(), VariableValue::Integer(*count));
            format!(
                "{{materialize}} {{n_random_characters($n)}} {} from your deck.",
                predicate_serializer::serialize_cost_constraint_only(predicate, bindings)
            )
        }
        StandardEffect::MultiplyYourEnergy { multiplier } => {
            strings::multiply_energy_effect(*multiplier).to_string()
        }
        StandardEffect::CopyNextPlayed { matching, times } => {
            if let Some(count) = times {
                bindings.insert("n".to_string(), VariableValue::Integer(*count));
            }
            format!(
                "copy the next {} you play {{this_turn_times($n)}}.",
                predicate_serializer::predicate_base_text(matching, bindings)
            )
        }
        StandardEffect::Copy { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::copy_target(target))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::disable_activated_abilities(target))
        }
        StandardEffect::DrawMatchingCard { predicate } => {
            let target = predicate_serializer::serialize_card_predicate(predicate, bindings);
            format!("{}.", strings::draw_matching_from_deck(target))
        }
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => {
                let target = predicate_serializer::predicate_base_text(matching, bindings);
                format!("{}.", strings::trigger_judgment_of_each(target))
            }
            CollectionExpression::Exactly(1) => {
                let target = predicate_serializer::serialize_predicate(matching, bindings);
                format!("{}.", strings::trigger_judgment_of(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(matching, bindings);
                format!("{}.", strings::trigger_judgment_of_n(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(matching, bindings);
                format!("{}.", strings::trigger_judgment_of(target))
            }
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            strings::judgment_phase_at_end_of_turn_effect().to_string()
        }
        StandardEffect::TakeExtraTurn => strings::take_extra_turn_effect().to_string(),
        StandardEffect::YouWinTheGame => strings::you_win_the_game_effect().to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::abandon_and_gain_energy_for_spark(target))
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::abandon_at_end_of_turn(target))
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::banish_when_leaves_play(target))
        }
        StandardEffect::DissolveCharactersQuantity { target, quantity } => {
            let target = predicate_serializer::serialize_predicate_plural(target, bindings);
            let quantity = serialize_for_count_expression(quantity, bindings);
            format!("{}.", strings::dissolve_all_with_cost_lte_quantity(target, quantity))
        }
        StandardEffect::PreventDissolveThisTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            format!("{}.", strings::prevent_dissolve_this_turn(target))
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, gains, for_each } => {
            let target = predicate_serializer::serialize_predicate(target, bindings);
            let for_each_text =
                predicate_serializer::serialize_for_each_predicate(for_each, bindings);
            format!(
                "{}.",
                strings::gains_spark_until_next_main_for_each(target, gains.0, for_each_text)
            )
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            strings::gain_twice_energy_instead_effect().to_string()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => {
            let target = predicate_serializer::serialize_card_predicate(target, bindings);
            format!("{}.", strings::materialize_from_void(target))
        }
        StandardEffect::ThenMaterializeIt => strings::then_materialize_it_effect().to_string(),
        StandardEffect::NoEffect => strings::no_effect().to_string(),
        StandardEffect::OpponentPaysCost { cost } => {
            let cost = cost_serializer::serialize_cost(cost, bindings);
            format!("{}.", strings::opponent_pays_cost(cost))
        }
        StandardEffect::PayCost { cost } => {
            let cost = cost_serializer::serialize_cost(cost, bindings);
            format!("{}.", strings::pay_cost_effect(cost))
        }
        StandardEffect::SpendAllEnergyDissolveEnemy => {
            strings::spend_all_energy_dissolve_effect().to_string()
        }
        StandardEffect::SpendAllEnergyDrawAndDiscard => {
            strings::spend_all_energy_draw_discard_effect().to_string()
        }
    }
}

/// Serializes an effect using the default event context.
pub fn serialize_effect(effect: &Effect, bindings: &mut VariableBindings) -> String {
    serialize_effect_with_context(effect, bindings, AbilityContext::Event)
}

/// Serializes an effect with explicit ability context.
///
/// The context determines joining behavior for mandatory effect lists:
/// - Triggered: use `, then` (e.g., "{Judgment} Draw {cards($c)}, then discard
///   {cards($d)}.")
/// - Event: use `. ` (e.g., "Draw {cards($c)}. Discard {cards($d)}.")
pub fn serialize_effect_with_context(
    effect: &Effect,
    bindings: &mut VariableBindings,
    context: AbilityContext,
) -> String {
    match effect {
        Effect::Effect(standard_effect) => serialize_standard_effect(standard_effect, bindings),
        Effect::WithOptions(options) => {
            let mut result = String::new();
            if let Some(condition) = &options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition, bindings));
                result.push(' ');
            }
            let needs_lowercase = options.optional || options.trigger_cost.is_some();
            if options.optional {
                result.push_str("you may ");
            }
            if let Some(trigger_cost) = &options.trigger_cost {
                let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost, bindings);
                let cost_str = if options.optional {
                    serializer_utils::lowercase_leading_keyword(&cost_str)
                } else {
                    cost_str
                };
                result.push_str(&format!("{} to ", cost_str));
            }
            let effect_str = serialize_standard_effect(&options.effect, bindings);
            let effect_str = if needs_lowercase {
                serializer_utils::lowercase_leading_keyword(&effect_str)
            } else {
                effect_str
            };
            result.push_str(&effect_str);
            result
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let has_condition = effects.first().and_then(|e| e.condition.as_ref()).is_some();
            let all_have_trigger_cost = effects.iter().all(|e| e.trigger_cost.is_some());
            if all_optional && all_have_trigger_cost && !effects.is_empty() {
                let effect_strings: Vec<String> = effects
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let s = serialize_standard_effect(&e.effect, bindings)
                            .trim_end_matches('.')
                            .to_string();
                        if i == 0 {
                            serializer_utils::lowercase_leading_keyword(&s)
                        } else {
                            s
                        }
                    })
                    .collect();
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(
                            condition, bindings,
                        ));
                        result.push(' ');
                    }
                }
                result.push_str("you may ");
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost, bindings);
                    let cost_str = serializer_utils::lowercase_leading_keyword(&cost_str);
                    result.push_str(&format!("{} to ", cost_str));
                }
                result.push_str(&format!("{}.", effect_strings.join(" and ")));
                result
            } else if !all_optional && all_have_trigger_cost && !effects.is_empty() {
                let effect_strings: Vec<String> = effects
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let s = serialize_standard_effect(&e.effect, bindings)
                            .trim_end_matches('.')
                            .to_string();
                        if i == 0 {
                            serializer_utils::lowercase_leading_keyword(&s)
                        } else {
                            s
                        }
                    })
                    .collect();
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(
                            condition, bindings,
                        ));
                        result.push(' ');
                    }
                }
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    result.push_str(&format!(
                        "{} to ",
                        cost_serializer::serialize_trigger_cost(trigger_cost, bindings)
                    ));
                }
                result.push_str(&format!("{}.", effect_strings.join(" and ")));
                result
            } else if all_optional && !effects.is_empty() {
                let effect_strings: Vec<String> = effects
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let s = serialize_standard_effect(&e.effect, bindings)
                            .trim_end_matches('.')
                            .to_string();
                        if i == 0 {
                            serializer_utils::lowercase_leading_keyword(&s)
                        } else {
                            s
                        }
                    })
                    .collect();
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(
                            condition, bindings,
                        ));
                        result.push(' ');
                    }
                }
                result.push_str(&format!("you may {}.", effect_strings.join(", then ")));
                result
            } else {
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(
                            condition, bindings,
                        ));
                        result.push(' ');
                    }
                }
                // For mandatory effect lists, use different joining based on context
                if context == AbilityContext::Triggered {
                    let effect_strings: Vec<String> = effects
                        .iter()
                        .map(|e| {
                            serialize_standard_effect(&e.effect, bindings)
                                .trim_end_matches('.')
                                .to_string()
                        })
                        .collect();
                    result.push_str(&format!("{}.", effect_strings.join(", then ")));
                } else {
                    let effect_strings: Vec<String> = effects
                        .iter()
                        .map(|e| {
                            let s = serialize_standard_effect(&e.effect, bindings);
                            let s = s.trim_end_matches('.');
                            format!("{}.", serializer_utils::capitalize_first_letter(s))
                        })
                        .collect();
                    result.push_str(&effect_strings.join(" "));
                }
                result
            }
        }
        Effect::ListWithOptions(list_with_options) => {
            let mut result = String::new();
            if let Some(condition) = &list_with_options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition, bindings));
                result.push(' ');
            }
            let has_shared_trigger_cost = list_with_options.trigger_cost.is_some();
            // Check if this is an optional action (parsed from "you may X to Y and Z")
            // by checking if ALL effects are optional AND there's a shared trigger_cost.
            // When all effects are optional with a shared cost, output "you may" once
            // at the start instead of for each effect.
            let all_effects_optional =
                list_with_options.effects.iter().all(|e| e.optional && e.trigger_cost.is_none());
            let is_optional_with_shared_cost = has_shared_trigger_cost && all_effects_optional;
            if is_optional_with_shared_cost {
                result.push_str("you may ");
            }
            if let Some(trigger_cost) = &list_with_options.trigger_cost {
                let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost, bindings);
                let cost_str = if is_optional_with_shared_cost {
                    serializer_utils::lowercase_leading_keyword(&cost_str)
                } else {
                    cost_str
                };
                result.push_str(&format!("{} to ", cost_str));
            }
            let effect_strings: Vec<String> = list_with_options
                .effects
                .iter()
                .map(|e| {
                    let mut effect_str = String::new();
                    // When using shared optional cost, don't add "you may" to individual effects
                    if e.optional && !is_optional_with_shared_cost {
                        effect_str.push_str("you may ");
                    }
                    if let Some(trigger_cost) = &e.trigger_cost {
                        effect_str.push_str(&format!(
                            "{} to ",
                            cost_serializer::serialize_trigger_cost(trigger_cost, bindings)
                        ));
                    }
                    if let Some(condition) = &e.condition {
                        effect_str.push_str(&condition_serializer::serialize_condition(
                            condition, bindings,
                        ));
                        effect_str.push(' ');
                    }
                    effect_str.push_str(
                        serialize_standard_effect(&e.effect, bindings).trim_end_matches('.'),
                    );
                    effect_str
                })
                .collect();
            // Use " and " when effects share a common trigger cost
            let joiner = if has_shared_trigger_cost { " and " } else { ", then " };
            result.push_str(&format!("{}.", effect_strings.join(joiner)));
            result
        }
        Effect::Modal(choices) => {
            let mut result = "{choose_one}".to_string();
            for (index, choice) in choices.iter().enumerate() {
                result.push('\n');
                result.push_str("{bullet} ");
                let (cost_var, var_name) =
                    if index == 0 { ("{energy($e1)}", "e1") } else { ("{energy($e2)}", "e2") };
                bindings.insert(var_name.to_string(), VariableValue::Integer(choice.energy_cost.0));
                result.push_str(&format!(
                    "{}: {}",
                    cost_var,
                    serializer_utils::capitalize_first_letter(&serialize_effect_with_context(
                        &choice.effect,
                        bindings,
                        context
                    ))
                ));
            }
            result
        }
    }
}

pub fn serialize_for_count_expression(
    quantity_expression: &QuantityExpression,
    bindings: &mut VariableBindings,
) -> String {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => {
            predicate_serializer::serialize_for_each_predicate(predicate, bindings).to_string()
        }
        QuantityExpression::PlayedThisTurn(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_played_this_turn(base).to_string()
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::Character) => {
            strings::ally_abandoned_this_turn().to_string()
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_abandoned_this_turn(serializer_utils::subtype_to_phrase(
                *subtype,
            ))
            .to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::Character) => {
            strings::ally_abandoned().to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_abandoned(serializer_utils::subtype_to_phrase(*subtype))
                .to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character) => {
            strings::ally_returned().to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_returned(serializer_utils::subtype_to_phrase(*subtype))
                .to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_returned(base).to_string()
        }
        QuantityExpression::AbandonedThisTurn(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_abandoned_this_turn(base).to_string()
        }
        QuantityExpression::AbandonedThisWay(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_abandoned(base).to_string()
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => strings::energy_spent().to_string(),
        QuantityExpression::CardsDrawnThisTurn(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_drawn_this_turn(base).to_string()
        }
        QuantityExpression::DiscardedThisTurn(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_discarded_this_turn(base).to_string()
        }
        QuantityExpression::DissolvedThisTurn(predicate) => {
            let base = predicate_serializer::card_predicate_base_phrase(predicate);
            strings::card_predicate_dissolved_this_turn(base).to_string()
        }
    }
}

fn serialize_allied_card_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allied {subtype($t)}".to_string()
        }
        _ => {
            format!("allied {}", predicate_serializer::card_predicate_base_text(card_predicate))
        }
    }
}

/// Serialize an allied card predicate in plural form for counting contexts.
fn serialize_allied_card_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
) -> String {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
            "allied {@plural subtype($t)}".to_string()
        }
        _ => {
            format!(
                "allied {}",
                predicate_serializer::card_predicate_base_text_plural(card_predicate)
            )
        }
    }
}

fn serialize_gains_reclaim(
    target: &Predicate,
    count: &CollectionExpression,
    this_turn: bool,
    cost: &Option<Energy>,
    bindings: &mut VariableBindings,
) -> String {
    let this_turn_suffix = if this_turn { " this turn" } else { "" };
    let (reclaim_directive, reclaim_suffix) = if let Some(energy_cost) = cost {
        bindings.insert("r".to_string(), VariableValue::Integer(energy_cost.0));
        ("{reclaim_for_cost($r)}", "")
    } else {
        ("{reclaim}", " equal to its cost")
    };

    match target {
        Predicate::It => {
            format!("it gains {}{}{}.", reclaim_directive, reclaim_suffix, this_turn_suffix)
        }
        Predicate::This => {
            format!("this card gains {}{}{}.", reclaim_directive, reclaim_suffix, this_turn_suffix)
        }
        Predicate::YourVoid(predicate) => {
            serialize_void_gains_reclaim(count, predicate, this_turn_suffix, cost, bindings)
        }
        _ => format!(
            "{} gains {}{}{}.",
            predicate_serializer::serialize_predicate(target, bindings),
            reclaim_directive,
            reclaim_suffix,
            this_turn_suffix
        ),
    }
}

fn serialize_void_gains_reclaim(
    count: &CollectionExpression,
    predicate: &CardPredicate,
    this_turn_suffix: &str,
    cost: &Option<Energy>,
    bindings: &mut VariableBindings,
) -> String {
    let (reclaim_directive, reclaim_suffix) = if let Some(energy_cost) = cost {
        bindings.insert("r".to_string(), VariableValue::Integer(energy_cost.0));
        ("{reclaim_for_cost($r)}", "")
    } else {
        ("{reclaim}", " equal to its cost")
    };
    let (reclaim_directive_plural, reclaim_suffix_plural) = if cost.is_some() {
        ("{reclaim_for_cost($r)}", "")
    } else {
        ("{reclaim}", " equal to their cost")
    };

    match count {
        CollectionExpression::Exactly(1) => {
            let predicate_text = if let CardPredicate::CharacterType(subtype) = predicate {
                bindings.insert("t".to_string(), VariableValue::Subtype(*subtype));
                "{@cap @a subtype($t)}".to_string()
            } else {
                serializer_utils::capitalize_first_letter(
                    &predicate_serializer::serialize_card_predicate(predicate, bindings)
                        .to_string(),
                )
            };
            format!(
                "{} in your void gains {}{}{}.",
                predicate_text, reclaim_directive, reclaim_suffix, this_turn_suffix
            )
        }
        CollectionExpression::Exactly(n) => format!(
            "{} {} in your void gain {}{}{}.",
            n,
            predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
            reclaim_directive_plural,
            reclaim_suffix_plural,
            this_turn_suffix
        ),
        CollectionExpression::All => format!(
            "all cards currently in your void gain {}{}{}.",
            reclaim_directive_plural, reclaim_suffix_plural, this_turn_suffix
        ),
        CollectionExpression::AllButOne => format!(
            "all but one {} in your void gain {}{}{}.",
            predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
            reclaim_directive_plural,
            reclaim_suffix_plural,
            this_turn_suffix
        ),
        CollectionExpression::UpTo(n) => format!(
            "up to {} {} in your void gain {}{}{}.",
            n,
            predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
            reclaim_directive_plural,
            reclaim_suffix_plural,
            this_turn_suffix
        ),
        CollectionExpression::AnyNumberOf => format!(
            "any number of {} in your void gain {}{}{}.",
            predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
            reclaim_directive_plural,
            reclaim_suffix_plural,
            this_turn_suffix
        ),
        CollectionExpression::OrMore(n) => format!(
            "{} or more {} in your void gain {}{}{}.",
            n,
            predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
            reclaim_directive_plural,
            reclaim_suffix_plural,
            this_turn_suffix
        ),
        CollectionExpression::EachOther => format!(
            "Each other card in your void gains {}{}{}",
            reclaim_directive, reclaim_suffix, this_turn_suffix
        ),
    }
}
