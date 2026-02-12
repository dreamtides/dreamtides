use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;
use core_data::numerics::Energy;
use rlf::Phrase;
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

/// Serializes a standard effect to a fragment string without trailing
/// punctuation. Assembly-level code adds the period.
pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            static_ability_serializer::serialize_standard_static_ability(ability)
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            let effect_fragment = serialize_effect_fragment(&trigger.effect);
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                strings::create_trigger_until_end_of_turn_keyword(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    strings::capitalized_sentence(effect_fragment),
                )
                .to_string()
            } else {
                strings::create_trigger_until_end_of_turn(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    effect_fragment,
                )
                .to_string()
            }
        }
        StandardEffect::DrawCards { count } => strings::draw_cards_effect(*count).to_string(),
        StandardEffect::DrawCardsForEach { count, for_each } => {
            strings::draw_cards_for_each(*count, serialize_for_count_expression(for_each))
                .to_string()
        }
        StandardEffect::DiscardCards { count } => strings::discard_cards_effect(*count).to_string(),
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            strings::discard_chosen_from_enemy_hand(
                predicate_serializer::card_predicate_without_article(predicate),
            )
            .to_string()
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            strings::discard_chosen_from_enemy_hand_then_draw(
                predicate_serializer::card_predicate_without_article(predicate),
            )
            .to_string()
        }
        StandardEffect::GainEnergy { gains } => strings::gain_energy_effect(gains.0).to_string(),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => {
                strings::gain_energy_equal_to_that_cost_effect().to_string()
            }
            Predicate::This => strings::gain_energy_equal_to_this_cost_effect().to_string(),
            _ => strings::gain_energy_equal_to_cost(predicate_serializer::serialize_predicate(
                target,
            ))
            .to_string(),
        },
        StandardEffect::GainEnergyForEach { gains, for_each } => strings::gain_energy_for_each(
            gains.0,
            predicate_serializer::for_each_predicate_phrase(for_each),
        )
        .to_string(),
        StandardEffect::GainPoints { gains } => strings::gain_points_effect(gains.0).to_string(),
        StandardEffect::GainPointsForEach { gain, for_count } => {
            strings::gain_points_for_each(gain.0, serialize_for_count_expression(for_count))
                .to_string()
        }
        StandardEffect::LosePoints { loses } => strings::lose_points_effect(loses.0).to_string(),
        StandardEffect::EnemyGainsPoints { count } => {
            strings::opponent_gains_points_effect(*count).to_string()
        }
        StandardEffect::EnemyGainsPointsEqualToItsSpark => {
            strings::opponent_gains_points_equal_spark(strings::this_character()).to_string()
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
            strings::gains_spark(predicate_serializer::serialize_predicate(target), gains.0)
                .to_string()
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            strings::have_each_gain_spark(serialize_allied_card_predicate(each), gains.0)
                .to_string()
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            strings::each_gains_spark_equal_to(
                serialize_allied_card_predicate(each),
                serialize_allied_card_predicate_plural(for_each),
            )
            .to_string()
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            strings::gains_spark_for_each(
                predicate_serializer::serialize_predicate(target),
                gains.0,
                serialize_for_count_expression(for_quantity),
            )
            .to_string()
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            strings::spark_of_each_becomes(serialize_allied_card_predicate(matching), spark.0)
                .to_string()
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            strings::put_deck_into_void_effect(*count).to_string()
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                strings::put_from_void_on_top_of_deck(
                    predicate_serializer::serialize_card_predicate_phrase(matching),
                )
                .to_string()
            } else {
                strings::put_up_to_from_void_on_top_of_deck(
                    *count,
                    predicate_serializer::serialize_card_predicate_plural_phrase(matching),
                )
                .to_string()
            }
        }
        StandardEffect::Counterspell { target } => {
            if matches!(target, Predicate::That | Predicate::It) {
                strings::prevent_that_card_effect().to_string()
            } else {
                strings::prevent_played_target(predicate_serializer::predicate_base_phrase(target))
                    .to_string()
            }
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            strings::prevent_unless_pays(
                predicate_serializer::predicate_base_phrase(target),
                cost_serializer::serialize_cost(cost),
            )
            .to_string()
        }
        StandardEffect::GainControl { target } => {
            strings::gain_control_of(predicate_serializer::serialize_predicate(target)).to_string()
        }
        StandardEffect::DissolveCharacter { target } => {
            strings::dissolve_target(predicate_serializer::serialize_predicate(target)).to_string()
        }
        StandardEffect::DissolveCharactersCount { target, count } => match count {
            CollectionExpression::All => {
                strings::dissolve_all(predicate_serializer::serialize_predicate_plural(target))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => strings::dissolve_exactly(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::UpTo(n) => strings::dissolve_up_to(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::AnyNumberOf => strings::dissolve_any_number_of(
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            _ => strings::dissolve_single(predicate_serializer::serialize_predicate(target))
                .to_string(),
        },
        StandardEffect::BanishCharacter { target } => {
            strings::banish_target(predicate_serializer::serialize_predicate(target)).to_string()
        }
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => strings::banish_any_number_of(
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::All => {
                strings::banish_all(predicate_serializer::serialize_predicate_plural(target))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => strings::banish_exactly(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::UpTo(n) => {
                strings::banish_up_to(*n, predicate_serializer::serialize_predicate_plural(target))
                    .to_string()
            }
            _ => strings::banish_single(predicate_serializer::serialize_predicate(target))
                .to_string(),
        },
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            strings::banish_cards_from_enemy_void_effect(*count).to_string()
        }
        StandardEffect::BanishEnemyVoid => strings::banish_enemy_void_effect().to_string(),
        StandardEffect::BanishThenMaterialize { target, count } => match count {
            CollectionExpression::Exactly(1) => strings::banish_then_materialize_it(
                predicate_serializer::serialize_predicate(target),
            )
            .to_string(),
            CollectionExpression::AnyNumberOf => strings::banish_then_materialize_any_number(
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            CollectionExpression::UpTo(n) => strings::banish_then_materialize_up_to(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            _ => strings::banish_then_materialize_them(predicate_serializer::serialize_predicate(
                target,
            ))
            .to_string(),
        },
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            strings::banish_until_leaves(
                predicate_serializer::serialize_predicate(target),
                predicate_serializer::predicate_base_phrase(until_leaves),
            )
            .to_string()
        }
        StandardEffect::BanishUntilNextMain { target } => {
            strings::banish_until_next_main(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::Discover { predicate } => strings::discover_target(
            predicate_serializer::serialize_card_predicate_phrase(predicate),
        )
        .to_string(),
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            strings::discover_and_materialize(
                predicate_serializer::serialize_card_predicate_phrase(predicate),
            )
            .to_string()
        }
        StandardEffect::MaterializeCharacter { target } => {
            strings::materialize_target(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            strings::materialize_at_end_of_turn(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    strings::materialize_copy_of(predicate_serializer::serialize_predicate(target))
                        .to_string()
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => strings::materialize_n_copies_of(
                    *n,
                    predicate_serializer::serialize_predicate(target),
                )
                .to_string(),
                (_, QuantityExpression::Matching(predicate)) => {
                    strings::materialize_copies_equal_to_matching(
                        predicate_serializer::serialize_predicate(target),
                        predicate_serializer::serialize_predicate_plural(predicate),
                    )
                    .to_string()
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    strings::materialize_copies_equal_to_energy(
                        predicate_serializer::serialize_predicate(target),
                    )
                    .to_string()
                }
                (_, quantity_expr) => strings::materialize_copies_equal_to_quantity(
                    predicate_serializer::serialize_predicate(target),
                    serialize_for_count_expression(quantity_expr),
                )
                .to_string(),
            }
        }
        StandardEffect::MaterializeFigments { count, figment } => strings::materialize_target(
            strings::n_figments(*count, serializer_utils::figment_to_phrase(*figment)),
        )
        .to_string(),
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, figment } => {
            let figment_text =
                strings::n_figments(*count, serializer_utils::figment_to_phrase(*figment));
            match quantity {
                QuantityExpression::PlayedThisTurn(_) => {
                    strings::materialize_figments_for_each_played(figment_text).to_string()
                }
                QuantityExpression::Matching(predicate) => strings::materialize_figments_for_each(
                    figment_text,
                    predicate_serializer::for_each_predicate_phrase(predicate),
                )
                .to_string(),
                _ => strings::materialize_figments_for_each_quantity(
                    figment_text,
                    serialize_for_count_expression(quantity),
                )
                .to_string(),
            }
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                strings::return_any_character_to_hand().to_string()
            }
            Predicate::Another(CardPredicate::Character) => {
                strings::return_ally_to_hand().to_string()
            }
            Predicate::This => strings::return_this_to_hand().to_string(),
            _ => strings::return_to_hand(predicate_serializer::serialize_predicate(target))
                .to_string(),
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            let target = match target {
                Predicate::YourVoid(card_predicate) => {
                    predicate_serializer::serialize_card_predicate_phrase(card_predicate)
                }
                _ => predicate_serializer::serialize_predicate(target),
            };
            strings::return_from_void_to_hand(target).to_string()
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => {
            strings::return_up_to_events_from_void_effect(*count).to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            strings::reclaim_target(predicate_serializer::serialize_predicate(target)).to_string()
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            strings::reclaim_random(predicate_serializer::card_predicate_without_article(predicate))
                .to_string()
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            strings::put_on_top_of_enemy_deck(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            strings::each_player_discards_effect(*count).to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            strings::each_player_abandons(predicate_serializer::serialize_card_predicate_phrase(
                matching,
            ))
            .to_string()
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count } => {
            strings::each_player_shuffles_and_draws_effect(*count).to_string()
        }
        StandardEffect::MaterializeCollection { target, count } => match (target, count) {
            (Predicate::Them, CollectionExpression::All) => {
                strings::materialize_them(strings::pronoun_them()).to_string()
            }
            (_, CollectionExpression::All) => {
                strings::materialize_all(predicate_serializer::serialize_predicate_plural(target))
                    .to_string()
            }
            (_, CollectionExpression::AnyNumberOf) => strings::materialize_any_number_of(
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            (_, CollectionExpression::UpTo(n)) => strings::materialize_up_to(
                *n,
                predicate_serializer::serialize_predicate_plural(target),
            )
            .to_string(),
            _ => strings::materialize_single(predicate_serializer::serialize_predicate(target))
                .to_string(),
        },
        StandardEffect::MaterializeRandomFromDeck { count, predicate } => {
            strings::materialize_random_from_deck(
                *count,
                predicate_serializer::serialize_cost_constraint_only(predicate),
            )
            .to_string()
        }
        StandardEffect::MultiplyYourEnergy { multiplier } => {
            strings::multiply_energy_effect(*multiplier).to_string()
        }
        StandardEffect::CopyNextPlayed { matching, times } => strings::copy_next_played(
            predicate_serializer::predicate_base_phrase(matching),
            times.unwrap_or(1),
        )
        .to_string(),
        StandardEffect::Copy { target } => {
            strings::copy_target(predicate_serializer::serialize_predicate(target)).to_string()
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            strings::disable_activated_abilities(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::DrawMatchingCard { predicate } => strings::draw_matching_from_deck(
            predicate_serializer::serialize_card_predicate_phrase(predicate),
        )
        .to_string(),
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => strings::trigger_judgment_of_each(
                predicate_serializer::predicate_base_phrase(matching),
            )
            .to_string(),
            CollectionExpression::Exactly(1) => {
                strings::trigger_judgment_of(predicate_serializer::serialize_predicate(matching))
                    .to_string()
            }
            CollectionExpression::Exactly(n) => strings::trigger_judgment_of_n(
                *n,
                predicate_serializer::serialize_predicate_plural(matching),
            )
            .to_string(),
            _ => strings::trigger_judgment_of(predicate_serializer::serialize_predicate(matching))
                .to_string(),
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            strings::judgment_phase_at_end_of_turn_effect().to_string()
        }
        StandardEffect::TakeExtraTurn => strings::take_extra_turn_effect().to_string(),
        StandardEffect::YouWinTheGame => strings::you_win_the_game_effect().to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            strings::abandon_and_gain_energy_for_spark(predicate_serializer::serialize_predicate(
                target,
            ))
            .to_string()
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            strings::abandon_at_end_of_turn(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            strings::banish_when_leaves_play(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::DissolveCharactersQuantity { target, quantity } => {
            strings::dissolve_all_with_cost_lte_quantity(
                predicate_serializer::serialize_predicate_plural(target),
                serialize_for_count_expression(quantity),
            )
            .to_string()
        }
        StandardEffect::PreventDissolveThisTurn { target } => {
            strings::prevent_dissolve_this_turn(predicate_serializer::serialize_predicate(target))
                .to_string()
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, gains, for_each } => {
            strings::gains_spark_until_next_main_for_each(
                predicate_serializer::serialize_predicate(target),
                gains.0,
                predicate_serializer::for_each_predicate_phrase(for_each),
            )
            .to_string()
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            strings::gain_twice_energy_instead_effect().to_string()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => strings::materialize_from_void(
            predicate_serializer::serialize_card_predicate_phrase(target),
        )
        .to_string(),
        StandardEffect::ThenMaterializeIt => {
            strings::then_materialize_it_effect(strings::pronoun_it()).to_string()
        }
        StandardEffect::NoEffect => strings::no_effect().to_string(),
        StandardEffect::OpponentPaysCost { cost } => {
            strings::opponent_pays_cost(cost_serializer::serialize_cost(cost)).to_string()
        }
        StandardEffect::PayCost { cost } => {
            strings::pay_cost_effect(cost_serializer::serialize_cost(cost)).to_string()
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
    let period = strings::period_suffix().to_string();
    match effect {
        Effect::Effect(standard_effect) => {
            let fragment = serialize_standard_effect(standard_effect);
            format!("{fragment}{period}")
        }
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
            result.push_str(&serialize_standard_effect(&options.effect));
            result.push_str(&period);
            result
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let has_condition = effects.first().and_then(|e| e.condition.as_ref()).is_some();
            let all_have_trigger_cost = effects.iter().all(|e| e.trigger_cost.is_some());
            let and_join = strings::and_joiner().to_string();
            let then_join = strings::then_joiner().to_string();
            if all_optional && all_have_trigger_cost && !effects.is_empty() {
                let effect_strings: Vec<String> =
                    effects.iter().map(|e| serialize_standard_effect(&e.effect)).collect();
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
                let effect_strings: Vec<String> =
                    effects.iter().map(|e| serialize_standard_effect(&e.effect)).collect();
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
                let effect_strings: Vec<String> =
                    effects.iter().map(|e| serialize_standard_effect(&e.effect)).collect();
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
                    let effect_strings: Vec<String> =
                        effects.iter().map(|e| serialize_standard_effect(&e.effect)).collect();
                    result.push_str(&effect_strings.join(&then_join));
                    result.push_str(&period);
                } else {
                    let effect_strings: Vec<String> = effects
                        .iter()
                        .map(|e| {
                            strings::capitalized_sentence(serialize_standard_effect(&e.effect))
                                .to_string()
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
                    effect_str.push_str(&serialize_standard_effect(&e.effect));
                    effect_str
                })
                .collect();
            let joiner = if has_shared_trigger_cost {
                strings::and_joiner().to_string()
            } else {
                strings::then_joiner().to_string()
            };
            result.push_str(&effect_strings.join(&joiner));
            result.push_str(&period);
            result
        }
        Effect::Modal(choices) => {
            let mut result = strings::choose_one().to_string();
            for choice in choices {
                result.push('\n');
                let energy_cost = strings::energy(choice.energy_cost.0);
                let effect_text = serialize_effect_with_context(&choice.effect, context);
                result.push_str(&strings::modal_choice_line(energy_cost, effect_text).to_string());
            }
            result
        }
    }
}

/// Serializes a quantity expression to a "for each" clause string.
pub fn serialize_for_count_expression(quantity_expression: &QuantityExpression) -> Phrase {
    match quantity_expression {
        QuantityExpression::Matching(predicate) => {
            predicate_serializer::for_each_predicate_phrase(predicate)
        }
        QuantityExpression::PlayedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_played_this_turn(base)
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::Character) => {
            strings::ally_abandoned_this_turn()
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_abandoned_this_turn(serializer_utils::subtype_to_phrase(
                *subtype,
            ))
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::Character) => strings::ally_abandoned(),
        QuantityExpression::AbandonedThisWay(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_abandoned(serializer_utils::subtype_to_phrase(*subtype))
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character) => {
            strings::ally_returned()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::CharacterType(subtype)) => {
            strings::allied_subtype_returned(serializer_utils::subtype_to_phrase(*subtype))
        }
        QuantityExpression::ReturnedToHandThisWay(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_returned(base)
        }
        QuantityExpression::AbandonedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_abandoned_this_turn(base)
        }
        QuantityExpression::AbandonedThisWay(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_abandoned(base)
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => strings::energy_spent(),
        QuantityExpression::CardsDrawnThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_drawn_this_turn(base)
        }
        QuantityExpression::DiscardedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_discarded_this_turn(base)
        }
        QuantityExpression::DissolvedThisTurn(predicate) => {
            let base = predicate_serializer::base_card_phrase(predicate);
            strings::card_predicate_dissolved_this_turn(base)
        }
    }
}

/// Serializes an effect as a periodless fragment for embedding in compound
/// phrases. Strips the trailing period from the full effect rendering.
fn serialize_effect_fragment(effect: &Effect) -> String {
    let rendered = serialize_effect(effect);
    rendered.strip_suffix(&strings::period_suffix().to_string()).unwrap_or(&rendered).to_string()
}

fn serialize_allied_card_predicate(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            strings::allied_card_with_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        _ => strings::allied_card_with_base(predicate_serializer::base_card_text(card_predicate)),
    }
}

/// Serialize an allied card predicate in plural form for counting contexts.
fn serialize_allied_card_predicate_plural(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            strings::allied_card_with_subtype_plural(serializer_utils::subtype_to_phrase(*subtype))
        }
        _ => strings::allied_card_with_base_plural(predicate_serializer::base_card_text_plural(
            card_predicate,
        )),
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
            let antecedent = strings::pronoun_it();
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::it_gains_reclaim_for_cost_this_turn(antecedent, energy_cost.0)
                        .to_string()
                } else {
                    strings::it_gains_reclaim_for_cost(antecedent, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::it_gains_reclaim_equal_cost_this_turn(antecedent).to_string()
            } else {
                strings::it_gains_reclaim_equal_cost(antecedent).to_string()
            }
        }
        Predicate::This => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::this_card_gains_reclaim_for_cost_this_turn(energy_cost.0).to_string()
                } else {
                    strings::this_card_gains_reclaim_for_cost(energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::this_card_gains_reclaim_equal_cost_this_turn().to_string()
            } else {
                strings::this_card_gains_reclaim_equal_cost().to_string()
            }
        }
        Predicate::YourVoid(predicate) => {
            serialize_void_gains_reclaim(count, predicate, this_turn, cost)
        }
        _ => {
            let target = predicate_serializer::serialize_predicate(target);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::target_gains_reclaim_for_cost_this_turn(target, energy_cost.0)
                        .to_string()
                } else {
                    strings::target_gains_reclaim_for_cost(target, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::target_gains_reclaim_equal_cost_this_turn(target).to_string()
            } else {
                strings::target_gains_reclaim_equal_cost(target).to_string()
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
                    strings::void_single_gains_reclaim_for_cost_this_turn(
                        predicate_text,
                        energy_cost.0,
                    )
                    .to_string()
                } else {
                    strings::void_single_gains_reclaim_for_cost(predicate_text, energy_cost.0)
                        .to_string()
                }
            } else if this_turn {
                strings::void_single_gains_reclaim_equal_cost_this_turn(predicate_text).to_string()
            } else {
                strings::void_single_gains_reclaim_equal_cost(predicate_text).to_string()
            }
        }
        CollectionExpression::Exactly(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_exactly_n_gain_reclaim_for_cost_this_turn(*n, pred, energy_cost.0)
                        .to_string()
                } else {
                    strings::void_exactly_n_gain_reclaim_for_cost(*n, pred, energy_cost.0)
                        .to_string()
                }
            } else if this_turn {
                strings::void_exactly_n_gain_reclaim_equal_cost_this_turn(*n, pred).to_string()
            } else {
                strings::void_exactly_n_gain_reclaim_equal_cost(*n, pred).to_string()
            }
        }
        CollectionExpression::All => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_all_gain_reclaim_for_cost_this_turn(energy_cost.0).to_string()
                } else {
                    strings::void_all_gain_reclaim_for_cost(energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_all_gain_reclaim_equal_cost_this_turn().to_string()
            } else {
                strings::void_all_gain_reclaim_equal_cost().to_string()
            }
        }
        CollectionExpression::AllButOne => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_all_but_one_gain_reclaim_for_cost_this_turn(pred, energy_cost.0)
                        .to_string()
                } else {
                    strings::void_all_but_one_gain_reclaim_for_cost(pred, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_all_but_one_gain_reclaim_equal_cost_this_turn(pred).to_string()
            } else {
                strings::void_all_but_one_gain_reclaim_equal_cost(pred).to_string()
            }
        }
        CollectionExpression::UpTo(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_up_to_gain_reclaim_for_cost_this_turn(*n, pred, energy_cost.0)
                        .to_string()
                } else {
                    strings::void_up_to_gain_reclaim_for_cost(*n, pred, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_up_to_gain_reclaim_equal_cost_this_turn(*n, pred).to_string()
            } else {
                strings::void_up_to_gain_reclaim_equal_cost(*n, pred).to_string()
            }
        }
        CollectionExpression::AnyNumberOf => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_any_number_gain_reclaim_for_cost_this_turn(pred, energy_cost.0)
                        .to_string()
                } else {
                    strings::void_any_number_gain_reclaim_for_cost(pred, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_any_number_gain_reclaim_equal_cost_this_turn(pred).to_string()
            } else {
                strings::void_any_number_gain_reclaim_equal_cost(pred).to_string()
            }
        }
        CollectionExpression::OrMore(n) => {
            let pred = predicate_serializer::serialize_card_predicate_plural_phrase(predicate);
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::void_or_more_gain_reclaim_for_cost_this_turn(*n, pred, energy_cost.0)
                        .to_string()
                } else {
                    strings::void_or_more_gain_reclaim_for_cost(*n, pred, energy_cost.0).to_string()
                }
            } else if this_turn {
                strings::void_or_more_gain_reclaim_equal_cost_this_turn(*n, pred).to_string()
            } else {
                strings::void_or_more_gain_reclaim_equal_cost(*n, pred).to_string()
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
