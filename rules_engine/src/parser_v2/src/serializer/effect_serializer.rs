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
        StandardEffect::DrawCards { count } => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::draw_cards_effect(0).to_string()
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            format!(
                "draw {{cards($c)}} for each {}.",
                serialize_for_count_expression(for_each, bindings)
            )
        }
        StandardEffect::DiscardCards { count } => {
            bindings.insert("d".to_string(), VariableValue::Integer(*count));
            strings::discard_cards_effect(0).to_string()
        }
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            format!(
                "discard a chosen {} from the opponent's hand.",
                predicate_serializer::serialize_card_predicate_without_article(predicate, bindings)
            )
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            bindings.insert("c".to_string(), VariableValue::Integer(1));
            format!(
                "discard a chosen {} from the opponent's hand. They draw {{cards($c)}}.",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        StandardEffect::GainEnergy { gains } => {
            bindings.insert("e".to_string(), VariableValue::Integer(gains.0));
            strings::gain_energy_effect(0).to_string()
        }
        StandardEffect::GainEnergyEqualToCost { target } => match target {
            Predicate::It | Predicate::That => {
                strings::gain_energy_equal_to_that_cost_effect().to_string()
            }
            Predicate::This => strings::gain_energy_equal_to_this_cost_effect().to_string(),
            _ => {
                format!(
                    "gain {{energy_symbol}} equal to {}'s cost.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
        },
        StandardEffect::GainEnergyForEach { gains, for_each } => {
            bindings.insert("e".to_string(), VariableValue::Integer(gains.0));
            format!(
                "gain {{energy($e)}} for each {}.",
                predicate_serializer::serialize_for_each_predicate(for_each, bindings)
            )
        }
        StandardEffect::GainPoints { gains } => {
            bindings.insert("p".to_string(), VariableValue::Integer(gains.0));
            strings::gain_points_effect(0).to_string()
        }
        StandardEffect::GainPointsForEach { gain, for_count } => {
            bindings.insert("p".to_string(), VariableValue::Integer(gain.0));
            format!(
                "gain {{points($p)}} for each {}.",
                serialize_for_count_expression(for_count, bindings)
            )
        }
        StandardEffect::LosePoints { loses } => {
            bindings.insert("p".to_string(), VariableValue::Integer(loses.0));
            strings::lose_points_effect(0).to_string()
        }
        StandardEffect::EnemyGainsPoints { count } => {
            bindings.insert("p".to_string(), VariableValue::Integer(*count));
            strings::opponent_gains_points_effect(0).to_string()
        }
        StandardEffect::EnemyGainsPointsEqualToItsSpark => {
            strings::opponent_gains_points_equal_spark().to_string()
        }
        StandardEffect::EnemyLosesPoints { count } => {
            bindings.insert("p".to_string(), VariableValue::Integer(*count));
            strings::opponent_loses_points_effect(0).to_string()
        }
        StandardEffect::Foresee { count } => {
            bindings.insert("f".to_string(), VariableValue::Integer(*count));
            strings::foresee_effect(0).to_string()
        }
        StandardEffect::Kindle { amount } => {
            bindings.insert("k".to_string(), VariableValue::Integer(amount.0));
            strings::kindle_effect(0).to_string()
        }
        StandardEffect::GainsReclaim { target, count, this_turn, cost } => {
            serialize_gains_reclaim(target, count, *this_turn, cost, bindings)
        }
        StandardEffect::GainsSpark { target, gains } => {
            bindings.insert("s".to_string(), VariableValue::Integer(gains.0));
            format!(
                "{} gains +{{$s}} spark.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            bindings.insert("s".to_string(), VariableValue::Integer(gains.0));
            format!(
                "have each {} gain +{{$s}} spark.",
                serialize_allied_card_predicate(each, bindings)
            )
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            format!(
                "each {} gains spark equal to the number of {}.",
                serialize_allied_card_predicate(each, bindings),
                serialize_allied_card_predicate_plural(for_each, bindings)
            )
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            bindings.insert("s".to_string(), VariableValue::Integer(gains.0));
            format!(
                "{} gains +{{$s}} spark for each {}.",
                predicate_serializer::serialize_predicate(target, bindings),
                serialize_for_count_expression(for_quantity, bindings)
            )
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            format!(
                "the spark of each {} becomes {{$s}}.",
                serialize_allied_card_predicate(matching, bindings)
            )
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            bindings.insert("v".to_string(), VariableValue::Integer(*count));
            strings::put_deck_into_void_effect(0).to_string()
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
                format!(
                    "{{prevent}} a played {}.",
                    predicate_serializer::predicate_base_text(target, bindings)
                )
            }
        }
        StandardEffect::CounterspellUnlessPaysCost { target, cost } => {
            format!(
                "{{prevent}} a played {} unless the opponent pays {}.",
                predicate_serializer::predicate_base_text(target, bindings),
                cost_serializer::serialize_cost(cost, bindings)
            )
        }
        StandardEffect::GainControl { target } => {
            format!(
                "gain control of {}.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::DissolveCharacter { target } => {
            format!("{{dissolve}} {}.", predicate_serializer::serialize_predicate(target, bindings))
        }
        StandardEffect::DissolveCharactersCount { target, count } => match count {
            CollectionExpression::All => {
                format!(
                    "{{dissolve}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "{{dissolve}} {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::UpTo(n) => {
                format!(
                    "{{dissolve}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::AnyNumberOf => {
                format!(
                    "{{dissolve}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            _ => {
                format!(
                    "{{dissolve}} {}.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
        },
        StandardEffect::BanishCharacter { target } => {
            format!("{{banish}} {}.", predicate_serializer::serialize_predicate(target, bindings))
        }
        StandardEffect::BanishCollection { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                format!(
                    "{{banish}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::All => {
                format!(
                    "{{banish}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "{{banish}} {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            CollectionExpression::UpTo(n) => {
                format!(
                    "{{banish}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            _ => {
                format!(
                    "{{banish}} {}.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
        },
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::banish_cards_from_enemy_void_effect(0).to_string()
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
            format!(
                "{{banish}} {} until {} leaves play.",
                predicate_serializer::serialize_predicate(target, bindings),
                predicate_serializer::predicate_base_text(until_leaves, bindings)
            )
        }
        StandardEffect::BanishUntilNextMain { target } => {
            format!(
                "{{banish}} {} until your next main phase.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::Discover { predicate } => {
            format!(
                "{{Discover}} {}.",
                predicate_serializer::serialize_card_predicate(predicate, bindings)
            )
        }
        StandardEffect::DiscoverAndThenMaterialize { predicate } => {
            format!(
                "{{Discover}} {} and {{materialize}} it.",
                predicate_serializer::serialize_card_predicate(predicate, bindings)
            )
        }
        StandardEffect::MaterializeCharacter { target } => {
            format!(
                "{{materialize}} {}.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => {
            format!(
                "{{materialize}} {} at end of turn.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
            match (count, quantity) {
                (1, QuantityExpression::Matching(_)) => {
                    format!(
                        "{{materialize}} a copy of {}.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
                (n, QuantityExpression::Matching(_)) if *n > 1 => {
                    format!(
                        "{{materialize}} {} copies of {}.",
                        n,
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
                (_, QuantityExpression::Matching(predicate)) => {
                    format!(
                        "{{materialize}} a number of copies of {} equal to the number of {}.",
                        predicate_serializer::serialize_predicate(target, bindings),
                        predicate_serializer::serialize_predicate_plural(predicate, bindings)
                    )
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    format!(
                        "{{materialize}} a number of copies of {} equal to the amount of {{energy_symbol}} spent.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
                (_, quantity_expr) => {
                    format!(
                        "{{materialize}} a number of copies of {} equal to the number of {}.",
                        predicate_serializer::serialize_predicate(target, bindings),
                        serialize_for_count_expression(quantity_expr, bindings)
                    )
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
                "{@a figment($g)}"
            } else {
                bindings.insert("n".to_string(), VariableValue::Integer(*count));
                "{n_figments($n, $g)}"
            };
            match quantity {
                QuantityExpression::PlayedThisTurn(_) => {
                    format!(
                        "{{materialize}} {} for each card you have played this turn.",
                        figment_text
                    )
                }
                QuantityExpression::Matching(predicate) => {
                    format!(
                        "{{materialize}} {} for each {}.",
                        figment_text,
                        predicate_serializer::serialize_for_each_predicate(predicate, bindings)
                    )
                }
                _ => {
                    format!(
                        "{{materialize}} {} for each {}.",
                        figment_text,
                        serialize_for_count_expression(quantity, bindings)
                    )
                }
            }
        }
        StandardEffect::ReturnToHand { target } => match target {
            Predicate::Any(CardPredicate::Character) => {
                "return an enemy or ally to hand.".to_string()
            }
            Predicate::Another(CardPredicate::Character) => "return an ally to hand.".to_string(),
            Predicate::This => "return this character to your hand.".to_string(),
            _ => {
                format!(
                    "return {} to hand.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
            }
        },
        StandardEffect::ReturnFromYourVoidToHand { target } => {
            // For YourVoid predicates, don't add "from your void" again since it's
            // already part of the predicate text
            match target {
                Predicate::YourVoid(card_predicate) => {
                    format!(
                        "return {} from your void to your hand.",
                        predicate_serializer::serialize_card_predicate(card_predicate, bindings)
                    )
                }
                _ => {
                    format!(
                        "return {} from your void to your hand.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::ReturnUpToCountFromYourVoidToHand { count, .. } => {
            bindings.insert("n".to_string(), VariableValue::Integer(*count));
            strings::return_up_to_events_from_void_effect(0).to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            format!("{{reclaim}} {}.", predicate_serializer::serialize_predicate(target, bindings))
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            format!(
                "{{reclaim}} a random {}.",
                predicate_serializer::serialize_card_predicate_without_article(predicate, bindings)
            )
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            format!(
                "put {} on top of the opponent's deck.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            bindings.insert("d".to_string(), VariableValue::Integer(*count));
            strings::each_player_discards_effect(0).to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            format!(
                "each player abandons {}.",
                predicate_serializer::serialize_card_predicate(matching, bindings)
            )
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count } => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::each_player_shuffles_and_draws_effect(0).to_string()
        }
        StandardEffect::MaterializeCollection { target, count } => match (target, count) {
            (Predicate::Them, CollectionExpression::All) => "{materialize} them.".to_string(),
            (_, CollectionExpression::All) => {
                format!(
                    "{{materialize}} all {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            (_, CollectionExpression::AnyNumberOf) => {
                format!(
                    "{{materialize}} any number of {}.",
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            (_, CollectionExpression::UpTo(n)) => {
                format!(
                    "{{materialize}} up to {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(target, bindings)
                )
            }
            _ => {
                format!(
                    "{{materialize}} {}.",
                    predicate_serializer::serialize_predicate(target, bindings)
                )
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
            bindings.insert("n".to_string(), VariableValue::Integer(*multiplier));
            strings::multiply_energy_effect(0).to_string()
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
            format!("copy {}.", predicate_serializer::serialize_predicate(target, bindings))
        }
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => {
            format!(
                "disable the activated abilities of {} while this character is in play.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::DrawMatchingCard { predicate } => {
            format!(
                "draw {} from your deck.",
                predicate_serializer::serialize_card_predicate(predicate, bindings)
            )
        }
        StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
            CollectionExpression::All => {
                format!(
                    "trigger the {{Judgment}} ability of each {}.",
                    predicate_serializer::predicate_base_text(matching, bindings)
                )
            }
            CollectionExpression::Exactly(1) => {
                format!(
                    "trigger the {{Judgment}} ability of {}.",
                    predicate_serializer::serialize_predicate(matching, bindings)
                )
            }
            CollectionExpression::Exactly(n) => {
                format!(
                    "trigger the {{Judgment}} ability of {} {}.",
                    n,
                    predicate_serializer::serialize_predicate_plural(matching, bindings)
                )
            }
            _ => {
                format!(
                    "trigger the {{Judgment}} ability of {}.",
                    predicate_serializer::serialize_predicate(matching, bindings)
                )
            }
        },
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            strings::judgment_phase_at_end_of_turn_effect().to_string()
        }
        StandardEffect::TakeExtraTurn => strings::take_extra_turn_effect().to_string(),
        StandardEffect::YouWinTheGame => strings::you_win_the_game_effect().to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            format!(
                "abandon {} and gain {{energy_symbol}} for each point of spark that character had.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::AbandonAtEndOfTurn { target } => {
            format!(
                "abandon {} at end of turn.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::BanishWhenLeavesPlay { target } => {
            format!(
                "{{banish}} {} when it leaves play.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::DissolveCharactersQuantity { target, quantity } => {
            format!(
                "{{dissolve}} all {} with cost less than or equal to the number of {}.",
                predicate_serializer::serialize_predicate_plural(target, bindings),
                serialize_for_count_expression(quantity, bindings)
            )
        }
        StandardEffect::PreventDissolveThisTurn { target } => {
            format!(
                "{} cannot be {{dissolved}} this turn.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, gains, for_each } => {
            bindings.insert("s".to_string(), VariableValue::Integer(gains.0));
            format!(
                "{} gains +{{$s}} spark until your next main phase for each {}.",
                predicate_serializer::serialize_predicate(target, bindings),
                predicate_serializer::serialize_for_each_predicate(for_each, bindings)
            )
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            strings::gain_twice_energy_instead_effect().to_string()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => {
            format!(
                "{{materialize}} {} from your void.",
                predicate_serializer::serialize_card_predicate(target, bindings)
            )
        }
        StandardEffect::ThenMaterializeIt => strings::then_materialize_it_effect().to_string(),
        StandardEffect::NoEffect => strings::no_effect().to_string(),
        StandardEffect::OpponentPaysCost { cost } => {
            format!("the opponent pays {}.", cost_serializer::serialize_cost(cost, bindings))
        }
        StandardEffect::PayCost { cost } => {
            format!("pay {}.", cost_serializer::serialize_cost(cost, bindings))
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
            predicate_serializer::serialize_for_each_predicate(predicate, bindings)
        }
        QuantityExpression::PlayedThisTurn(predicate) => {
            format!(
                "{} you have played this turn",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::Character) => {
            "ally abandoned this turn".to_string()
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::CharacterType(_)) => {
            "allied {subtype($t)} abandoned this turn".to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::Character) => {
            "ally abandoned".to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::CharacterType(_)) => {
            "allied {subtype($t)} abandoned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character) => {
            "ally returned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::CharacterType(_)) => {
            "allied {subtype($t)} returned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(predicate) => {
            format!("{} returned", predicate_serializer::card_predicate_base_text(predicate))
        }
        QuantityExpression::AbandonedThisTurn(predicate) => {
            format!(
                "{} abandoned this turn",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        QuantityExpression::AbandonedThisWay(predicate) => {
            format!("{} abandoned", predicate_serializer::card_predicate_base_text(predicate))
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => "{energy_symbol} spent".to_string(),
        QuantityExpression::CardsDrawnThisTurn(predicate) => {
            format!(
                "{} you have drawn this turn",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        QuantityExpression::DiscardedThisTurn(predicate) => {
            format!(
                "{} you have discarded this turn",
                predicate_serializer::card_predicate_base_text(predicate)
            )
        }
        QuantityExpression::DissolvedThisTurn(predicate) => {
            format!(
                "{} which dissolved this turn",
                predicate_serializer::card_predicate_base_text(predicate)
            )
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
                    &predicate_serializer::serialize_card_predicate(predicate, bindings),
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
