use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;
use core_data::numerics::Energy;
use strings::strings;

use crate::serializer::{
    condition_serializer, cost_serializer, predicate_serializer, serializer_utils,
    static_ability_serializer, trigger_serializer,
};

/// Context for effect serialization to determine joining behavior.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AbilityContext {
    /// Triggered ability - use `, then` for mandatory effect lists.
    Triggered,
    /// Event or other ability - use `. ` for mandatory effect lists.
    #[default]
    Event,
}

/// Serializes a standard effect to its text representation.
pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            let base = static_ability_serializer::serialize_standard_static_ability(ability);
            if base.ends_with('.') {
                base
            } else {
                format!("{}{}", base, strings::period_suffix())
            }
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                strings::create_trigger_until_end_of_turn_keyword(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    strings::capitalized_sentence(serialize_effect(&trigger.effect)),
                )
                .to_string()
            } else {
                strings::create_trigger_until_end_of_turn(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    serialize_effect(&trigger.effect),
                )
                .to_string()
            }
        }
        StandardEffect::DrawCards { count } => strings::draw_cards_effect(*count).to_string(),
        StandardEffect::DrawCardsForEach { count, for_each } => {
            let target = serialize_for_count_expression(for_each);
            format!("{}.", strings::draw_cards_for_each(*count, target))
        }
        StandardEffect::DiscardCards { count } => strings::discard_cards_effect(*count).to_string(),
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            let target = predicate_serializer::card_predicate_without_article(predicate);
            format!("{}.", strings::discard_chosen_from_enemy_hand(target))
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            let target = predicate_serializer::card_predicate_without_article(predicate);
            format!("{}.", strings::discard_chosen_from_enemy_hand_then_draw(target))
        }
        StandardEffect::GainEnergy { gains } => strings::gain_energy_effect(gains.0).to_string(),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => {
                strings::gain_energy_equal_to_that_cost_effect().to_string()
            }
            Predicate::This => strings::gain_energy_equal_to_this_cost_effect().to_string(),
            _ => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::gain_energy_equal_to_cost(target))
            }
        },
        StandardEffect::GainEnergyForEach { gains, for_each } => {
            let target = predicate_serializer::for_each_predicate_phrase(for_each);
            format!("{}.", strings::gain_energy_for_each(gains.0, target))
        }
        StandardEffect::GainPoints { gains } => strings::gain_points_effect(gains.0).to_string(),
        StandardEffect::GainPointsForEach { gain, for_count } => {
            let target = serialize_for_count_expression(for_count);
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
            serialize_gains_reclaim(target, count, *this_turn, cost)
        }
        StandardEffect::GainsSpark { target, gains } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::gains_spark(target, gains.0))
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            let each_text = serialize_allied_card_predicate(each);
            format!("{}.", strings::have_each_gain_spark(each_text, gains.0))
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            let each_text = serialize_allied_card_predicate(each);
            let count_of = serialize_allied_card_predicate_plural(for_each);
            format!("{}.", strings::each_gains_spark_equal_to(each_text, count_of))
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            let target = predicate_serializer::serialize_predicate(target);
            let quantity = serialize_for_count_expression(for_quantity);
            format!("{}.", strings::gains_spark_for_each(target, gains.0, quantity))
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            let each_text = serialize_allied_card_predicate(matching);
            format!("{}.", strings::spark_of_each_becomes(each_text, spark.0))
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            strings::put_deck_into_void_effect(*count).to_string()
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                let target = predicate_serializer::serialize_card_predicate_phrase(matching);
                format!("{}.", strings::put_from_void_on_top_of_deck(target))
            } else {
                let target = predicate_serializer::serialize_card_predicate_plural_phrase(matching);
                format!("{}.", strings::put_up_to_from_void_on_top_of_deck(*count, target))
            }
        }
        StandardEffect::Counterspell { target } => {
            if matches!(target, Predicate::That | Predicate::It) {
                strings::prevent_that_card_effect().to_string()
            } else {
                let target = predicate_serializer::predicate_base_phrase(target);
                format!("{}.", strings::prevent_played_target(target))
            }
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            let target = predicate_serializer::predicate_base_phrase(target);
            let cost = cost_serializer::serialize_cost(cost);
            format!("{}.", strings::prevent_unless_pays(target, cost))
        }
        StandardEffect::GainControl { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::gain_control_of(target))
        }
        StandardEffect::DissolveCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::dissolve_target(target))
        }
        StandardEffect::DissolveCharactersCount { target, count } => match count {
            CollectionExpression::All => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::dissolve_all(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::dissolve_exactly(*n, target))
            }
            CollectionExpression::UpTo(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::dissolve_up_to(*n, target))
            }
            CollectionExpression::AnyNumberOf => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::dissolve_any_number_of(target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::dissolve_single(target))
            }
        },
        StandardEffect::BanishCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::banish_target(target))
        }
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_any_number_of(target))
            }
            CollectionExpression::All => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_all(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_exactly(*n, target))
            }
            CollectionExpression::UpTo(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_up_to(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::banish_single(target))
            }
        },
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            strings::banish_cards_from_enemy_void_effect(*count).to_string()
        }
        StandardEffect::BanishEnemyVoid => strings::banish_enemy_void_effect().to_string(),
        StandardEffect::BanishThenMaterialize { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::banish_then_materialize_it(target))
            }
            CollectionExpression::AnyNumberOf => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_then_materialize_any_number(target))
            }
            CollectionExpression::UpTo(n) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::banish_then_materialize_up_to(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::banish_then_materialize_them(target))
            }
        },
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            let target = predicate_serializer::serialize_predicate(target);
            let until = predicate_serializer::predicate_base_phrase(until_leaves);
            format!("{}.", strings::banish_until_leaves(target, until))
        }
        StandardEffect::BanishUntilNextMain { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::banish_until_next_main(target))
        }
        StandardEffect::Discover { predicate } => {
            let target = predicate_serializer::serialize_card_predicate_phrase(predicate);
            format!("{}.", strings::discover_target(target))
        }
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            let target = predicate_serializer::serialize_card_predicate_phrase(predicate);
            format!("{}.", strings::discover_and_materialize(target))
        }
        StandardEffect::MaterializeCharacter { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::materialize_target(target))
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::materialize_at_end_of_turn(target))
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    let target = predicate_serializer::serialize_predicate(target);
                    format!("{}.", strings::materialize_copy_of(target))
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => {
                    let target = predicate_serializer::serialize_predicate(target);
                    format!("{}.", strings::materialize_n_copies_of(*n, target))
                }
                (_, QuantityExpression::Matching(predicate)) => {
                    let target = predicate_serializer::serialize_predicate(target);
                    let matching = predicate_serializer::serialize_predicate_plural(predicate);
                    format!("{}.", strings::materialize_copies_equal_to_matching(target, matching))
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    let target = predicate_serializer::serialize_predicate(target);
                    format!("{}.", strings::materialize_copies_equal_to_energy(target))
                }
                (_, quantity_expr) => {
                    let target = predicate_serializer::serialize_predicate(target);
                    let quantity = serialize_for_count_expression(quantity_expr);
                    format!("{}.", strings::materialize_copies_equal_to_quantity(target, quantity))
                }
            }
        }
        StandardEffect::MaterializeFigments { count, figment } => {
            let figment_phrase = serializer_utils::figment_to_phrase(*figment);
            format!("{}.", strings::materialize_target(strings::n_figments(*count, figment_phrase)))
        }
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, figment } => {
            let figment_text =
                strings::n_figments(*count, serializer_utils::figment_to_phrase(*figment))
                    .to_string();
            match quantity {
                QuantityExpression::PlayedThisTurn(_) => {
                    format!("{}.", strings::materialize_figments_for_each_played(figment_text))
                }
                QuantityExpression::Matching(predicate) => {
                    let for_each = predicate_serializer::for_each_predicate_phrase(predicate);
                    format!("{}.", strings::materialize_figments_for_each(figment_text, for_each))
                }
                _ => {
                    let quantity_text = serialize_for_count_expression(quantity);
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
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::return_to_hand(target))
            }
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            let target = match target {
                Predicate::YourVoid(card_predicate) => {
                    predicate_serializer::serialize_card_predicate_phrase(card_predicate)
                }
                _ => predicate_serializer::serialize_predicate(target),
            };
            format!("{}.", strings::return_from_void_to_hand(target))
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => {
            strings::return_up_to_events_from_void_effect(*count).to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::reclaim_target(target))
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            let target = predicate_serializer::card_predicate_without_article(predicate);
            format!("{}.", strings::reclaim_random(target))
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::put_on_top_of_enemy_deck(target))
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            strings::each_player_discards_effect(*count).to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            let target = predicate_serializer::serialize_card_predicate_phrase(matching);
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
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::materialize_all(target))
            }
            (_, CollectionExpression::AnyNumberOf) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::materialize_any_number_of(target))
            }
            (_, CollectionExpression::UpTo(n)) => {
                let target = predicate_serializer::serialize_predicate_plural(target);
                format!("{}.", strings::materialize_up_to(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(target);
                format!("{}.", strings::materialize_single(target))
            }
        },
        StandardEffect::MaterializeRandomFromDeck { count, predicate } => {
            let constraint = predicate_serializer::serialize_cost_constraint_only(predicate);
            format!("{}.", strings::materialize_random_from_deck(*count, constraint))
        }
        StandardEffect::MultiplyYourEnergy { multiplier } => {
            strings::multiply_energy_effect(*multiplier).to_string()
        }
        StandardEffect::CopyNextPlayed { matching, times } => {
            let target = predicate_serializer::predicate_base_phrase(matching);
            format!("{}.", strings::copy_next_played(target, times.unwrap_or(1)))
        }
        StandardEffect::Copy { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::copy_target(target))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::disable_activated_abilities(target))
        }
        StandardEffect::DrawMatchingCard { predicate } => {
            let target = predicate_serializer::serialize_card_predicate_phrase(predicate);
            format!("{}.", strings::draw_matching_from_deck(target))
        }
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => {
                let target = predicate_serializer::predicate_base_phrase(matching);
                format!("{}.", strings::trigger_judgment_of_each(target))
            }
            CollectionExpression::Exactly(1) => {
                let target = predicate_serializer::serialize_predicate(matching);
                format!("{}.", strings::trigger_judgment_of(target))
            }
            CollectionExpression::Exactly(n) => {
                let target = predicate_serializer::serialize_predicate_plural(matching);
                format!("{}.", strings::trigger_judgment_of_n(*n, target))
            }
            _ => {
                let target = predicate_serializer::serialize_predicate(matching);
                format!("{}.", strings::trigger_judgment_of(target))
            }
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            strings::judgment_phase_at_end_of_turn_effect().to_string()
        }
        StandardEffect::TakeExtraTurn => strings::take_extra_turn_effect().to_string(),
        StandardEffect::YouWinTheGame => strings::you_win_the_game_effect().to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::abandon_and_gain_energy_for_spark(target))
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::abandon_at_end_of_turn(target))
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::banish_when_leaves_play(target))
        }
        StandardEffect::DissolveCharactersQuantity { target, quantity } => {
            let target = predicate_serializer::serialize_predicate_plural(target);
            let quantity = serialize_for_count_expression(quantity);
            format!("{}.", strings::dissolve_all_with_cost_lte_quantity(target, quantity))
        }
        StandardEffect::PreventDissolveThisTurn { target } => {
            let target = predicate_serializer::serialize_predicate(target);
            format!("{}.", strings::prevent_dissolve_this_turn(target))
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, gains, for_each } => {
            let target = predicate_serializer::serialize_predicate(target);
            let for_each_text = predicate_serializer::for_each_predicate_phrase(for_each);
            format!(
                "{}.",
                strings::gains_spark_until_next_main_for_each(target, gains.0, for_each_text)
            )
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            strings::gain_twice_energy_instead_effect().to_string()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => {
            let target = predicate_serializer::serialize_card_predicate_phrase(target);
            format!("{}.", strings::materialize_from_void(target))
        }
        StandardEffect::ThenMaterializeIt => strings::then_materialize_it_effect().to_string(),
        StandardEffect::NoEffect => strings::no_effect().to_string(),
        StandardEffect::OpponentPaysCost { cost } => {
            let cost = cost_serializer::serialize_cost(cost);
            format!("{}.", strings::opponent_pays_cost(cost))
        }
        StandardEffect::PayCost { cost } => {
            let cost = cost_serializer::serialize_cost(cost);
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
pub fn serialize_effect(effect: &Effect) -> String {
    serialize_effect_with_context(effect, AbilityContext::Event)
}

/// Serializes an effect with explicit ability context.
///
/// The context determines joining behavior for mandatory effect lists:
/// - Triggered: use `, then` (e.g., "{Judgment} Draw {cards($c)}, then discard
///   {cards($d)}.")
/// - Event: use `. ` (e.g., "Draw {cards($c)}. Discard {cards($d)}.")
pub fn serialize_effect_with_context(effect: &Effect, context: AbilityContext) -> String {
    match effect {
        Effect::Effect(standard_effect) => serialize_standard_effect(standard_effect),
        Effect::WithOptions(options) => {
            let mut result = String::new();
            if let Some(condition) = &options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition));
                result.push(' ');
            }
            if options.optional {
                result.push_str(&strings::you_may_prefix().to_string());
            }
            if let Some(trigger_cost) = &options.trigger_cost {
                let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                result.push_str(&strings::cost_to_connector(cost_str).to_string());
            }
            let effect_str = serialize_standard_effect(&options.effect);
            result.push_str(&effect_str);
            result
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let has_condition = effects.first().and_then(|e| e.condition.as_ref()).is_some();
            let all_have_trigger_cost = effects.iter().all(|e| e.trigger_cost.is_some());
            let and_join = strings::and_joiner().to_string();
            let then_join = strings::then_joiner().to_string();
            let period = strings::period_suffix().to_string();
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
                result.push_str(&strings::you_may_prefix().to_string());
                if let Some(trigger_cost) = &effects[0].trigger_cost {
                    let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                    result.push_str(&strings::cost_to_connector(cost_str).to_string());
                }
                result.push_str(&effect_strings.join(&and_join));
                result.push_str(&period);
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
                    let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                    result.push_str(&strings::cost_to_connector(cost_str).to_string());
                }
                result.push_str(&effect_strings.join(&and_join));
                result.push_str(&period);
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
                result.push_str(&strings::you_may_prefix().to_string());
                result.push_str(&effect_strings.join(&then_join));
                result.push_str(&period);
                result
            } else {
                let mut result = String::new();
                if has_condition {
                    if let Some(condition) = &effects[0].condition {
                        result.push_str(&condition_serializer::serialize_condition(condition));
                        result.push(' ');
                    }
                }
                if context == AbilityContext::Triggered {
                    let effect_strings: Vec<String> = effects
                        .iter()
                        .map(|e| {
                            serialize_standard_effect(&e.effect).trim_end_matches('.').to_string()
                        })
                        .collect();
                    result.push_str(&effect_strings.join(&then_join));
                    result.push_str(&period);
                } else {
                    let effect_strings: Vec<String> = effects
                        .iter()
                        .map(|e| {
                            let s = serialize_standard_effect(&e.effect);
                            let s = s.trim_end_matches('.');
                            strings::capitalized_sentence(s).to_string()
                        })
                        .collect();
                    let sentence_join = strings::sentence_joiner().to_string();
                    result.push_str(&effect_strings.join(&sentence_join));
                    result.push_str(&period);
                }
                result
            }
        }
        Effect::ListWithOptions(list_with_options) => {
            let mut result = String::new();
            if let Some(condition) = &list_with_options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition));
                result.push(' ');
            }
            let has_shared_trigger_cost = list_with_options.trigger_cost.is_some();
            let all_effects_optional =
                list_with_options.effects.iter().all(|e| e.optional && e.trigger_cost.is_none());
            let is_optional_with_shared_cost = has_shared_trigger_cost && all_effects_optional;
            if is_optional_with_shared_cost {
                result.push_str(&strings::you_may_prefix().to_string());
            }
            if let Some(trigger_cost) = &list_with_options.trigger_cost {
                let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                result.push_str(&strings::cost_to_connector(cost_str).to_string());
            }
            let effect_strings: Vec<String> = list_with_options
                .effects
                .iter()
                .map(|e| {
                    let mut effect_str = String::new();
                    if e.optional && !is_optional_with_shared_cost {
                        effect_str.push_str(&strings::you_may_prefix().to_string());
                    }
                    if let Some(trigger_cost) = &e.trigger_cost {
                        let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                        effect_str.push_str(&strings::cost_to_connector(cost_str).to_string());
                    }
                    if let Some(condition) = &e.condition {
                        effect_str.push_str(&condition_serializer::serialize_condition(condition));
                        effect_str.push(' ');
                    }
                    effect_str.push_str(serialize_standard_effect(&e.effect).trim_end_matches('.'));
                    effect_str
                })
                .collect();
            let joiner = if has_shared_trigger_cost {
                strings::and_joiner().to_string()
            } else {
                strings::then_joiner().to_string()
            };
            result.push_str(&effect_strings.join(&joiner));
            result.push_str(&strings::period_suffix().to_string());
            result
        }
        Effect::Modal(choices) => {
            let cost_effect_sep = strings::cost_effect_separator().to_string();
            let mut result = "{choose_one}".to_string();
            for choice in choices {
                result.push('\n');
                result.push_str("{bullet} ");
                let cost_text = format!("{{energy({})}}", choice.energy_cost.0);
                let effect_text = serialize_effect_with_context(&choice.effect, context);
                let capitalized = strings::capitalized_sentence(effect_text).to_string();
                result.push_str(&format!("{cost_text}{cost_effect_sep}{capitalized}"));
            }
            result
        }
    }
}

/// Serializes a quantity expression to a "for each" clause string.
pub fn serialize_for_count_expression(quantity_expression: &QuantityExpression) -> String {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => {
            predicate_serializer::for_each_predicate_phrase(predicate).to_string()
        }
        QuantityExpression::PlayedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
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
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_returned(base).to_string()
        }
        QuantityExpression::AbandonedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_abandoned_this_turn(base).to_string()
        }
        QuantityExpression::AbandonedThisWay(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_abandoned(base).to_string()
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => strings::energy_spent().to_string(),
        QuantityExpression::CardsDrawnThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_drawn_this_turn(base).to_string()
        }
        QuantityExpression::DiscardedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_discarded_this_turn(base).to_string()
        }
        QuantityExpression::DissolvedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_dissolved_this_turn(base).to_string()
        }
    }
}

fn serialize_allied_card_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            strings::allied_card_with_subtype(serializer_utils::subtype_to_phrase(*subtype))
                .to_string()
        }
        _ => strings::allied_card_with_base(predicate_serializer::base_card_text(card_predicate))
            .to_string(),
    }
}

/// Serialize an allied card predicate in plural form for counting contexts.
fn serialize_allied_card_predicate_plural(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            strings::allied_card_with_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
                .to_string()
        }
        _ => strings::allied_card_with_base_plural(predicate_serializer::base_card_text_plural(
            card_predicate,
        ))
        .to_string(),
    }
}

fn serialize_gains_reclaim(
    target: &Predicate,
    count: &CollectionExpression,
    this_turn: bool,
    cost: &Option<Energy>,
) -> String {
    match target {
        Predicate::It => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!("{}.", strings::it_gains_reclaim_for_cost_this_turn(energy_cost.0))
                } else {
                    format!("{}.", strings::it_gains_reclaim_for_cost(energy_cost.0))
                }
            } else if this_turn {
                format!("{}.", strings::it_gains_reclaim_equal_cost_this_turn())
            } else {
                format!("{}.", strings::it_gains_reclaim_equal_cost())
            }
        }
        Predicate::This => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::this_card_gains_reclaim_for_cost_this_turn(energy_cost.0)
                    )
                } else {
                    format!("{}.", strings::this_card_gains_reclaim_for_cost(energy_cost.0))
                }
            } else if this_turn {
                format!("{}.", strings::this_card_gains_reclaim_equal_cost_this_turn())
            } else {
                format!("{}.", strings::this_card_gains_reclaim_equal_cost())
            }
        }
        Predicate::YourVoid(predicate) => {
            serialize_void_gains_reclaim(count, predicate, this_turn, cost)
        }
        _ => {
            let target = predicate_serializer::serialize_predicate(target);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::target_gains_reclaim_for_cost_this_turn(target, energy_cost.0)
                    )
                } else {
                    format!("{}.", strings::target_gains_reclaim_for_cost(target, energy_cost.0))
                }
            } else if this_turn {
                format!("{}.", strings::target_gains_reclaim_equal_cost_this_turn(target))
            } else {
                format!("{}.", strings::target_gains_reclaim_equal_cost(target))
            }
        }
    }
}

fn serialize_void_gains_reclaim(
    count: &CollectionExpression,
    predicate: &CardPredicate,
    this_turn: bool,
    cost: &Option<Energy>,
) -> String {
    match count {
        CollectionExpression::Exactly(1) => {
            let predicate_text = if let CardPredicate::CharacterType(subtype) = predicate {
                strings::capitalized_sentence(strings::predicate_with_indefinite_article(
                    strings::subtype(serializer_utils::subtype_to_phrase(*subtype)),
                ))
                .to_string()
            } else {
                strings::capitalized_sentence(
                    predicate_serializer::serialize_card_predicate_phrase(predicate),
                )
                .to_string()
            };
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_single_gains_reclaim_for_cost_this_turn(
                            predicate_text,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_single_gains_reclaim_for_cost(predicate_text, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!(
                    "{}.",
                    strings::void_single_gains_reclaim_equal_cost_this_turn(predicate_text)
                )
            } else {
                format!("{}.", strings::void_single_gains_reclaim_equal_cost(predicate_text))
            }
        }
        CollectionExpression::Exactly(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_exactly_n_gain_reclaim_for_cost_this_turn(
                            *n,
                            pred,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_exactly_n_gain_reclaim_for_cost(*n, pred, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!("{}.", strings::void_exactly_n_gain_reclaim_equal_cost_this_turn(*n, pred))
            } else {
                format!("{}.", strings::void_exactly_n_gain_reclaim_equal_cost(*n, pred))
            }
        }
        CollectionExpression::All => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!("{}.", strings::void_all_gain_reclaim_for_cost_this_turn(energy_cost.0))
                } else {
                    format!("{}.", strings::void_all_gain_reclaim_for_cost(energy_cost.0))
                }
            } else if this_turn {
                format!("{}.", strings::void_all_gain_reclaim_equal_cost_this_turn())
            } else {
                format!("{}.", strings::void_all_gain_reclaim_equal_cost())
            }
        }
        CollectionExpression::AllButOne => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_all_but_one_gain_reclaim_for_cost_this_turn(
                            pred,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_all_but_one_gain_reclaim_for_cost(pred, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!("{}.", strings::void_all_but_one_gain_reclaim_equal_cost_this_turn(pred))
            } else {
                format!("{}.", strings::void_all_but_one_gain_reclaim_equal_cost(pred))
            }
        }
        CollectionExpression::UpTo(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_up_to_gain_reclaim_for_cost_this_turn(
                            *n,
                            pred,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_up_to_gain_reclaim_for_cost(*n, pred, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!("{}.", strings::void_up_to_gain_reclaim_equal_cost_this_turn(*n, pred))
            } else {
                format!("{}.", strings::void_up_to_gain_reclaim_equal_cost(*n, pred))
            }
        }
        CollectionExpression::AnyNumberOf => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_any_number_gain_reclaim_for_cost_this_turn(
                            pred,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_any_number_gain_reclaim_for_cost(pred, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!("{}.", strings::void_any_number_gain_reclaim_equal_cost_this_turn(pred))
            } else {
                format!("{}.", strings::void_any_number_gain_reclaim_equal_cost(pred))
            }
        }
        CollectionExpression::OrMore(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    format!(
                        "{}.",
                        strings::void_or_more_gain_reclaim_for_cost_this_turn(
                            *n,
                            pred,
                            energy_cost.0
                        )
                    )
                } else {
                    format!(
                        "{}.",
                        strings::void_or_more_gain_reclaim_for_cost(*n, pred, energy_cost.0)
                    )
                }
            } else if this_turn {
                format!("{}.", strings::void_or_more_gain_reclaim_equal_cost_this_turn(*n, pred))
            } else {
                format!("{}.", strings::void_or_more_gain_reclaim_equal_cost(*n, pred))
            }
        }
        CollectionExpression::EachOther => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_each_other_gains_reclaim_for_cost_this_turn(energy_cost.0)
                        .to_string()
                } else {
                    strings::void_each_other_gains_reclaim_for_cost(energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_each_other_gains_reclaim_equal_cost_this_turn().to_string()
            } else {
                strings::void_each_other_gains_reclaim_equal_cost().to_string()
            }
        }
    }
}
