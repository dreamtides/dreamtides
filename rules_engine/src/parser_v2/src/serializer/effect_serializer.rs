use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::{Effect, EffectWithOptions};
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

/// Serializes a standard effect to a phrase without trailing punctuation.
/// Assembly-level code adds the period.
pub fn serialize_standard_effect(effect: &StandardEffect) -> Phrase {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            static_ability_serializer::serialize_standard_static_ability(ability)
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            let effect_fragment = serialize_effect_fragment(&trigger.effect);
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                strings::create_trigger_until_end_of_turn_keyword(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    effect_fragment,
                )
            } else {
                strings::create_trigger_until_end_of_turn(
                    trigger_serializer::serialize_trigger_event(&trigger.trigger),
                    effect_fragment,
                )
            }
        }
        StandardEffect::DrawCards { count } => strings::draw_cards_effect(*count),
        StandardEffect::DrawCardsForEach { count, for_each } => {
            strings::draw_cards_for_each(*count, serialize_for_count_expression(for_each))
        }
        StandardEffect::DiscardCards { count } => strings::discard_cards_effect(*count),
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            strings::discard_chosen_from_enemy_hand(
                predicate_serializer::card_predicate_without_article(predicate),
            )
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            strings::discard_chosen_from_enemy_hand_then_draw(
                predicate_serializer::card_predicate_without_article(predicate),
            )
        }
        StandardEffect::GainEnergy { gains } => strings::gain_energy_effect(gains.0),
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => strings::gain_energy_equal_to_that_cost_effect(),
            Predicate::This => strings::gain_energy_equal_to_this_cost_effect(),
            _ => strings::gain_energy_equal_to_cost(predicate_serializer::serialize_predicate(
                target,
            )),
        },
        StandardEffect::GainEnergyForEach { gains, for_each } => strings::gain_energy_for_each(
            gains.0,
            predicate_serializer::for_each_predicate_phrase(for_each),
        ),
        StandardEffect::GainPoints { gains } => strings::gain_points_effect(gains.0),
        StandardEffect::GainPointsForEach { gain, for_count } => {
            strings::gain_points_for_each(gain.0, serialize_for_count_expression(for_count))
        }
        StandardEffect::LosePoints { loses } => strings::lose_points_effect(loses.0),
        StandardEffect::EnemyGainsPoints { count } => strings::opponent_gains_points_effect(*count),
        StandardEffect::EnemyGainsPointsEqualToItsSpark => {
            strings::opponent_gains_points_equal_spark(strings::this_character())
        }
        StandardEffect::EnemyLosesPoints { count } => strings::opponent_loses_points_effect(*count),
        StandardEffect::Foresee { count } => strings::foresee_effect(*count),
        StandardEffect::Kindle { amount } => strings::kindle_effect(amount.0),
        StandardEffect::GainsReclaim { target, count, this_turn, cost } => {
            serialize_gains_reclaim(target, count, *this_turn, cost)
        }
        StandardEffect::GainsSpark { target, gains } => {
            strings::gains_spark(predicate_serializer::serialize_predicate(target), gains.0)
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            strings::have_each_gain_spark(serialize_allied_card_predicate(each), gains.0)
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            strings::each_gains_spark_equal_to(
                serialize_allied_card_predicate(each),
                serialize_allied_card_predicate(for_each),
            )
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            strings::gains_spark_for_each(
                predicate_serializer::serialize_predicate(target),
                gains.0,
                serialize_for_count_expression(for_quantity),
            )
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            strings::spark_of_each_becomes(serialize_allied_card_predicate(matching), spark.0)
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            strings::put_deck_into_void_effect(*count)
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                strings::put_from_void_on_top_of_deck(
                    predicate_serializer::serialize_card_predicate_phrase(matching),
                )
            } else {
                strings::put_up_to_from_void_on_top_of_deck(
                    *count,
                    predicate_serializer::serialize_card_predicate_phrase(matching),
                )
            }
        }
        StandardEffect::Counterspell { target } => {
            if matches!(target, Predicate::That | Predicate::It) {
                strings::prevent_that_card_effect()
            } else {
                strings::prevent_played_target(predicate_serializer::predicate_base_phrase(target))
            }
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            strings::prevent_unless_pays(
                predicate_serializer::predicate_base_phrase(target),
                cost_serializer::serialize_cost(cost),
            )
        }
        StandardEffect::GainControl { target } => {
            strings::gain_control_of(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DissolveCharacter { target } => {
            strings::dissolve_target(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DissolveCharactersCount { target, count } => {
            strings::dissolve_collection(serialize_collection_target(count, target))
        }
        StandardEffect::BanishCharacter { target } => {
            strings::banish_target(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::BanishCollection { target, count } => {
            strings::banish_collection_target(serialize_collection_target(count, target))
        }
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            strings::banish_cards_from_enemy_void_effect(*count)
        }
        StandardEffect::BanishEnemyVoid => strings::banish_enemy_void_effect(),
        StandardEffect::BanishThenMaterialize { target, count } => match count {
            CollectionExpression::Exactly(1) => strings::banish_then_materialize_it(
                predicate_serializer::serialize_predicate(target),
            ),
            CollectionExpression::AnyNumberOf => strings::banish_then_materialize_any_number(
                predicate_serializer::serialize_predicate(target),
            ),
            CollectionExpression::UpTo(n) => strings::banish_then_materialize_up_to(
                *n,
                predicate_serializer::serialize_predicate(target),
            ),
            _ => strings::banish_then_materialize_them(predicate_serializer::serialize_predicate(
                target,
            )),
        },
        StandardEffect::BanishCharacterUntilLeavesPlay { target, until_leaves } => {
            strings::banish_until_leaves(
                predicate_serializer::serialize_predicate(target),
                predicate_serializer::predicate_base_phrase(until_leaves),
            )
        }
        StandardEffect::BanishUntilNextMain { target } => {
            strings::banish_until_next_main(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::Discover { predicate } => strings::discover_target(
            predicate_serializer::serialize_card_predicate_phrase(predicate),
        ),
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            strings::discover_and_materialize(
                predicate_serializer::serialize_card_predicate_phrase(predicate),
            )
        }
        StandardEffect::MaterializeCharacter { target } => {
            strings::materialize_target(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            strings::materialize_at_end_of_turn(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    strings::materialize_copy_of(predicate_serializer::serialize_predicate(target))
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => strings::materialize_n_copies_of(
                    *n,
                    predicate_serializer::serialize_predicate(target),
                ),
                (_, QuantityExpression::Matching(predicate)) => {
                    strings::materialize_copies_equal_to_matching(
                        predicate_serializer::serialize_predicate(target),
                        predicate_serializer::serialize_predicate(predicate),
                    )
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    strings::materialize_copies_equal_to_energy(
                        predicate_serializer::serialize_predicate(target),
                    )
                }
                (_, quantity_expr) => strings::materialize_copies_equal_to_quantity(
                    predicate_serializer::serialize_predicate(target),
                    serialize_for_count_expression(quantity_expr),
                ),
            }
        }
        StandardEffect::MaterializeFigments { count, figment } => strings::materialize_target(
            strings::n_figments(*count, serializer_utils::figment_to_phrase(*figment)),
        ),
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, figment } => {
            let figment_text =
                strings::n_figments(*count, serializer_utils::figment_to_phrase(*figment));
            strings::materialize_figments_for_each_quantity(
                figment_text,
                serialize_for_count_expression(quantity),
            )
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => strings::return_any_character_to_hand(),
            Predicate::Another(CardPredicate::Character) => strings::return_ally_to_hand(),
            Predicate::This => strings::return_this_to_hand(),
            _ => strings::return_to_hand(predicate_serializer::serialize_predicate(target)),
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            let target = match target {
                Predicate::YourVoid(card_predicate) => {
                    predicate_serializer::serialize_card_predicate_phrase(card_predicate)
                }
                _ => predicate_serializer::serialize_predicate(target),
            };
            strings::return_from_void_to_hand(target)
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => {
            strings::return_up_to_events_from_void_effect(*count)
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            strings::reclaim_target(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            strings::reclaim_random(predicate_serializer::card_predicate_without_article(predicate))
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            strings::put_on_top_of_enemy_deck(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            strings::each_player_discards_effect(*count)
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            strings::each_player_abandons(predicate_serializer::serialize_card_predicate_phrase(
                matching,
            ))
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count } => {
            strings::each_player_shuffles_and_draws_effect(*count)
        }
        StandardEffect::MaterializeCollection { target, count } => {
            if matches!((target, count), (Predicate::Them, CollectionExpression::All)) {
                strings::materialize_them(strings::pronoun_them())
            } else {
                strings::materialize_collection_target(serialize_collection_target(count, target))
            }
        }
        StandardEffect::MaterializeRandomFromDeck { count, predicate } => {
            strings::materialize_random_from_deck(
                *count,
                predicate_serializer::serialize_cost_constraint_only(predicate),
            )
        }
        StandardEffect::MultiplyYourEnergy { multiplier } => {
            strings::multiply_energy_effect(*multiplier)
        }
        StandardEffect::CopyNextPlayed { matching, times } => strings::copy_next_played(
            predicate_serializer::predicate_base_phrase(matching),
            times.unwrap_or(1),
        ),
        StandardEffect::Copy { target } => {
            strings::copy_target(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            strings::disable_activated_abilities(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DrawMatchingCard { predicate } => strings::draw_matching_from_deck(
            predicate_serializer::serialize_card_predicate_phrase(predicate),
        ),
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => strings::trigger_judgment_of_each(
                predicate_serializer::predicate_base_phrase(matching),
            ),
            _ => strings::trigger_judgment_of_collection(serialize_collection_target(
                collection, matching,
            )),
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            strings::judgment_phase_at_end_of_turn_effect()
        }
        StandardEffect::TakeExtraTurn => strings::take_extra_turn_effect(),
        StandardEffect::YouWinTheGame => strings::you_win_the_game_effect(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            strings::abandon_and_gain_energy_for_spark(predicate_serializer::serialize_predicate(
                target,
            ))
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            strings::abandon_at_end_of_turn(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            strings::banish_when_leaves_play(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::DissolveCharactersQuantity { target, quantity } => {
            strings::dissolve_all_with_cost_lte_quantity(
                predicate_serializer::serialize_predicate(target),
                serialize_for_count_expression(quantity),
            )
        }
        StandardEffect::PreventDissolveThisTurn { target } => {
            strings::prevent_dissolve_this_turn(predicate_serializer::serialize_predicate(target))
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, gains, for_each } => {
            strings::gains_spark_until_next_main_for_each(
                predicate_serializer::serialize_predicate(target),
                gains.0,
                predicate_serializer::for_each_predicate_phrase(for_each),
            )
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            strings::gain_twice_energy_instead_effect()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => strings::materialize_from_void(
            predicate_serializer::serialize_card_predicate_phrase(target),
        ),
        StandardEffect::ThenMaterializeIt => {
            strings::then_materialize_it_effect(strings::pronoun_it())
        }
        StandardEffect::NoEffect => strings::no_effect(),
        StandardEffect::OpponentPaysCost { cost } => {
            strings::opponent_pays_cost(cost_serializer::serialize_cost(cost))
        }
        StandardEffect::PayCost { cost } => {
            strings::pay_cost_effect(cost_serializer::serialize_cost(cost))
        }
        StandardEffect::SpendAllEnergyDissolveEnemy => strings::spend_all_energy_dissolve_effect(),
        StandardEffect::SpendAllEnergyDrawAndDiscard => {
            strings::spend_all_energy_draw_discard_effect()
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
        Effect::Effect(standard_effect) => {
            strings::effect_with_period(serialize_standard_effect(standard_effect)).to_string()
        }
        Effect::WithOptions(options) => {
            let effect_text = serialize_standard_effect(&options.effect).to_string();
            let body = match (options.optional, &options.trigger_cost) {
                (true, Some(trigger_cost)) => {
                    let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                    strings::optional_cost_effect_body(cost_str, effect_text).to_string()
                }
                (true, None) => strings::optional_effect_body(effect_text).to_string(),
                (false, Some(trigger_cost)) => {
                    let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                    strings::cost_effect_body(cost_str, effect_text).to_string()
                }
                (false, None) => effect_text,
            };
            let with_period = strings::effect_with_period(body).to_string();
            if let Some(condition) = &options.condition {
                strings::condition_with_effect(
                    condition_serializer::serialize_condition(condition),
                    with_period,
                )
                .to_string()
            } else {
                with_period
            }
        }
        Effect::List(effects) => {
            let all_optional = effects.iter().all(|e| e.optional);
            let all_have_trigger_cost = effects.iter().all(|e| e.trigger_cost.is_some());
            let and_join = strings::and_joiner().to_string();
            let then_join = strings::then_joiner().to_string();
            let body = if all_optional && all_have_trigger_cost && !effects.is_empty() {
                let joined = join_effect_fragments(effects, &and_join);
                let cost_str =
                    effects[0].trigger_cost.as_ref().map(cost_serializer::serialize_trigger_cost);
                match cost_str {
                    Some(cost) => strings::optional_cost_effect_body(cost, joined).to_string(),
                    None => strings::optional_effect_body(joined).to_string(),
                }
            } else if !all_optional && all_have_trigger_cost && !effects.is_empty() {
                let joined = join_effect_fragments(effects, &and_join);
                let cost_str =
                    effects[0].trigger_cost.as_ref().map(cost_serializer::serialize_trigger_cost);
                match cost_str {
                    Some(cost) => strings::cost_effect_body(cost, joined).to_string(),
                    None => joined,
                }
            } else if all_optional && !effects.is_empty() {
                let joined = join_effect_fragments(effects, &then_join);
                strings::optional_effect_body(joined).to_string()
            } else if context == AbilityContext::Triggered {
                join_effect_fragments(effects, &then_join)
            } else {
                let separator = strings::sentence_separator().to_string();
                let sentences: Vec<String> = effects
                    .iter()
                    .map(|e| {
                        strings::capitalized_sentence_with_period(serialize_standard_effect(
                            &e.effect,
                        ))
                        .to_string()
                    })
                    .collect();
                return prepend_condition_from_list(effects, sentences.join(&separator));
            };
            let with_period = strings::effect_with_period(body).to_string();
            prepend_condition_from_list(effects, with_period)
        }
        Effect::ListWithOptions(list_with_options) => {
            let has_shared_trigger_cost = list_with_options.trigger_cost.is_some();
            let all_effects_optional =
                list_with_options.effects.iter().all(|e| e.optional && e.trigger_cost.is_none());
            let is_optional_with_shared_cost = has_shared_trigger_cost && all_effects_optional;
            let effect_strings: Vec<String> = list_with_options
                .effects
                .iter()
                .map(|e| {
                    let effect_text = serialize_standard_effect(&e.effect).to_string();
                    let with_condition = if let Some(condition) = &e.condition {
                        strings::per_effect_condition(
                            condition_serializer::serialize_condition(condition),
                            effect_text,
                        )
                        .to_string()
                    } else {
                        effect_text
                    };
                    let with_cost = if let Some(trigger_cost) = &e.trigger_cost {
                        let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                        strings::per_effect_cost(cost_str, with_condition).to_string()
                    } else {
                        with_condition
                    };
                    if e.optional && !is_optional_with_shared_cost {
                        strings::per_effect_optional(with_cost).to_string()
                    } else {
                        with_cost
                    }
                })
                .collect();
            let joiner = if has_shared_trigger_cost {
                strings::and_joiner().to_string()
            } else {
                strings::then_joiner().to_string()
            };
            let joined = effect_strings.join(&joiner);
            let body = if is_optional_with_shared_cost {
                match &list_with_options.trigger_cost {
                    Some(trigger_cost) => {
                        let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                        strings::optional_cost_effect_body(cost_str, joined).to_string()
                    }
                    None => strings::optional_effect_body(joined).to_string(),
                }
            } else if let Some(trigger_cost) = &list_with_options.trigger_cost {
                let cost_str = cost_serializer::serialize_trigger_cost(trigger_cost);
                strings::cost_effect_body(cost_str, joined).to_string()
            } else {
                joined
            };
            let with_period = strings::effect_with_period(body).to_string();
            if let Some(condition) = &list_with_options.condition {
                strings::condition_with_effect(
                    condition_serializer::serialize_condition(condition),
                    with_period,
                )
                .to_string()
            } else {
                with_period
            }
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
pub fn serialize_effect_fragment(effect: &Effect) -> String {
    let rendered = serialize_effect(effect);
    rendered.strip_suffix(&strings::period_suffix().to_string()).unwrap_or(&rendered).to_string()
}

/// Serializes a collection expression with a predicate target to produce the
/// appropriate quantified target phrase for use with verb phrases.
fn serialize_collection_target(collection: &CollectionExpression, target: &Predicate) -> Phrase {
    match collection {
        CollectionExpression::All => {
            strings::collection_all(predicate_serializer::serialize_predicate(target))
        }
        CollectionExpression::Exactly(1) => predicate_serializer::serialize_predicate(target),
        CollectionExpression::Exactly(n) => {
            strings::collection_exactly(*n, predicate_serializer::serialize_predicate(target))
        }
        CollectionExpression::UpTo(n) => {
            strings::collection_up_to(*n, predicate_serializer::serialize_predicate(target))
        }
        CollectionExpression::AnyNumberOf => {
            strings::collection_any_number_of(predicate_serializer::serialize_predicate(target))
        }
        _ => predicate_serializer::serialize_predicate(target),
    }
}

fn serialize_allied_card_predicate(card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::CharacterType(subtype) => {
            strings::allied_card_with_subtype(serializer_utils::subtype_to_phrase(*subtype))
        }
        _ => strings::allied_card_with_base(predicate_serializer::base_card_text(card_predicate)),
    }
}

/// Joins serialized effect fragments from an effect list using the given
/// joiner string.
fn join_effect_fragments(effects: &[EffectWithOptions], joiner: &str) -> String {
    let effect_strings: Vec<String> =
        effects.iter().map(|e| serialize_standard_effect(&e.effect).to_string()).collect();
    effect_strings.join(joiner)
}

/// Prepends a condition from the first effect in a list, if present.
fn prepend_condition_from_list(effects: &[EffectWithOptions], body: String) -> String {
    let condition = effects.first().and_then(|e| e.condition.as_ref());
    if let Some(condition) = condition {
        strings::condition_with_effect(condition_serializer::serialize_condition(condition), body)
            .to_string()
    } else {
        body
    }
}

fn serialize_gains_reclaim(
    target: &Predicate,
    count: &CollectionExpression,
    this_turn: bool,
    cost: &Option<Energy>,
) -> Phrase {
    match target {
        Predicate::It => {
            let antecedent = strings::pronoun_it();
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::it_gains_reclaim_for_cost_this_turn(antecedent, energy_cost.0)
                } else {
                    strings::it_gains_reclaim_for_cost(antecedent, energy_cost.0)
                }
            } else if this_turn {
                strings::it_gains_reclaim_equal_cost_this_turn(antecedent)
            } else {
                strings::it_gains_reclaim_equal_cost(antecedent)
            }
        }
        Predicate::This => {
            if let Some(energy_cost) = cost {
                if this_turn {
                    strings::this_card_gains_reclaim_for_cost_this_turn(energy_cost.0)
                } else {
                    strings::this_card_gains_reclaim_for_cost(energy_cost.0)
                }
            } else if this_turn {
                strings::this_card_gains_reclaim_equal_cost_this_turn()
            } else {
                strings::this_card_gains_reclaim_equal_cost()
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
                } else {
                    strings::target_gains_reclaim_for_cost(target, energy_cost.0)
                }
            } else if this_turn {
                strings::target_gains_reclaim_equal_cost_this_turn(target)
            } else {
                strings::target_gains_reclaim_equal_cost(target)
            }
        }
    }
}

fn serialize_void_gains_reclaim(
    count: &CollectionExpression,
    predicate: &CardPredicate,
    this_turn: bool,
    cost: &Option<Energy>,
) -> Phrase {
    let (subject, is_singular) = serialize_void_collection_subject(count, predicate);
    assemble_void_reclaim_effect(subject, is_singular, this_turn, cost)
}

/// Produces the subject phrase and singularity flag for a void collection
/// expression.
fn serialize_void_collection_subject(
    count: &CollectionExpression,
    predicate: &CardPredicate,
) -> (Phrase, bool) {
    match count {
        CollectionExpression::Exactly(1) => {
            let predicate_text = if let CardPredicate::CharacterType(subtype) = predicate {
                strings::predicate_with_indefinite_article(strings::subtype(
                    serializer_utils::subtype_to_phrase(*subtype),
                ))
            } else {
                predicate_serializer::serialize_card_predicate_phrase(predicate)
            };
            (strings::void_subject_single(predicate_text), true)
        }
        CollectionExpression::Exactly(n) => {
            let pred = predicate_serializer::serialize_card_predicate_phrase(predicate);
            (strings::void_subject_exactly(*n, pred), false)
        }
        CollectionExpression::All => (strings::void_subject_all(), false),
        CollectionExpression::AllButOne => {
            let pred = predicate_serializer::serialize_card_predicate_phrase(predicate);
            (strings::void_subject_all_but_one(pred), false)
        }
        CollectionExpression::UpTo(n) => {
            let pred = predicate_serializer::serialize_card_predicate_phrase(predicate);
            (strings::void_subject_up_to(*n, pred), false)
        }
        CollectionExpression::AnyNumberOf => {
            let pred = predicate_serializer::serialize_card_predicate_phrase(predicate);
            (strings::void_subject_any_number(pred), false)
        }
        CollectionExpression::OrMore(n) => {
            let pred = predicate_serializer::serialize_card_predicate_phrase(predicate);
            (strings::void_subject_or_more(*n, pred), false)
        }
        CollectionExpression::EachOther => (strings::void_subject_each_other(), true),
    }
}

/// Assembles a void-reclaim effect from subject, singularity, duration, and
/// cost parameters by dispatching to the appropriate assembly phrase.
fn assemble_void_reclaim_effect(
    subject: Phrase,
    is_singular: bool,
    this_turn: bool,
    cost: &Option<Energy>,
) -> Phrase {
    match (is_singular, this_turn, cost) {
        (true, false, Some(energy_cost)) => {
            strings::void_gains_reclaim_for_cost_singular(subject, energy_cost.0)
        }
        (true, false, None) => strings::void_gains_reclaim_equal_cost_singular(subject),
        (true, true, Some(energy_cost)) => {
            strings::void_gains_reclaim_for_cost_singular_this_turn(subject, energy_cost.0)
        }
        (true, true, None) => strings::void_gains_reclaim_equal_cost_singular_this_turn(subject),
        (false, false, Some(energy_cost)) => {
            strings::void_gains_reclaim_for_cost_plural(subject, energy_cost.0)
        }
        (false, false, None) => strings::void_gains_reclaim_equal_cost_plural(subject),
        (false, true, Some(energy_cost)) => {
            strings::void_gains_reclaim_for_cost_plural_this_turn(subject, energy_cost.0)
        }
        (false, true, None) => strings::void_gains_reclaim_equal_cost_plural_this_turn(subject),
    }
}
