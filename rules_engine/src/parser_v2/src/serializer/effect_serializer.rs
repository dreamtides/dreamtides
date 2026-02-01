use ability_data::collection_expression::CollectionExpression;
use ability_data::effect::Effect;
use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::quantity_expression_data::QuantityExpression;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;
use ability_data::variable_value::VariableValue;

use crate::serializer::{
    condition_serializer, cost_serializer, predicate_serializer, serializer_utils,
    static_ability_serializer, text_formatting, trigger_serializer,
};
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions;

pub fn serialize_standard_effect(
    effect: &StandardEffect,
    bindings: &mut VariableBindings,
) -> String {
    match effect {
        StandardEffect::CreateStaticAbilityUntilEndOfTurn { ability } => {
            static_ability_serializer::serialize_standard_static_ability(
                ability,
                bindings,
            )
        }
        StandardEffect::CreateTriggerUntilEndOfTurn { trigger } => {
            if matches!(trigger.trigger, TriggerEvent::Keywords(_)) {
                format!(
                    "until end of turn, {} {}",
                    trigger_serializer::serialize_trigger_event(& trigger.trigger,
                    bindings), serializer_utils::capitalize_first_letter(&
                    serialize_effect(& trigger.effect, bindings))
                )
            } else {
                format!(
                    "until end of turn, {}{}",
                    trigger_serializer::serialize_trigger_event(& trigger.trigger,
                    bindings), serialize_effect(& trigger.effect, bindings)
                )
            }
        }
        StandardEffect::DrawCards { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "draw {cards}.".to_string()
        }
        StandardEffect::DrawCardsForEach { count, for_each } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            format!(
                "draw {{cards}} for each {}.", serialize_for_count_expression(for_each,
                bindings)
            )
        }
        StandardEffect::DiscardCards { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "discards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "discard {discards}.".to_string()
        }
        StandardEffect::DiscardCardFromEnemyHand { predicate } => {
            format!(
                "discard a chosen {} from the opponent's hand.",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { predicate } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(1));
            }
            format!(
                "discard a chosen {} from the opponent's hand. They draw {{cards}}.",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        StandardEffect::GainEnergy { gains } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "e",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            "gain {e}.".to_string()
        }
        StandardEffect::GainEnergyEqualToCost { target } => {
            match target {
                Predicate::It | Predicate::That => {
                    "gain {energy-symbol} equal to that character's cost.".to_string()
                }
                Predicate::This => {
                    "gain {energy-symbol} equal to this character's cost.".to_string()
                }
                _ => {
                    format!(
                        "gain {{energy-symbol}} equal to {}'s cost.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::GainEnergyForEach { gains, for_each } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "e",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            format!(
                "gain {{e}} for each {}.",
                predicate_serializer::serialize_for_each_predicate(for_each, bindings)
            )
        }
        StandardEffect::GainPoints { gains } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "points",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            "gain {points}.".to_string()
        }
        StandardEffect::GainPointsForEach { gain, for_count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "points",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gain.0));
            }
            format!(
                "gain {{points}} for each {}.", serialize_for_count_expression(for_count,
                bindings)
            )
        }
        StandardEffect::LosePoints { loses } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "points",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(loses.0));
            }
            "you lose {points}.".to_string()
        }
        StandardEffect::EnemyGainsPoints { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "points",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "the opponent gains {points}.".to_string()
        }
        StandardEffect::EnemyGainsPointsEqualToItsSpark => {
            "the opponent gains points equal to its spark.".to_string()
        }
        StandardEffect::EnemyLosesPoints { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "points",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "the opponent loses {points}.".to_string()
        }
        StandardEffect::Foresee { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "foresee",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "{foresee}.".to_string()
        }
        StandardEffect::Kindle { amount } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "kindle",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(amount.0));
            }
            "{kindle}.".to_string()
        }
        StandardEffect::GainsReclaimUntilEndOfTurn { target, cost } => {
            match (target, cost) {
                (Predicate::It, None) => {
                    "it gains {reclaim} equal to its cost this turn.".to_string()
                }
                (_, Some(energy_cost)) => {
                    if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                        "reclaim-for-cost",
                    ) {
                        bindings
                            .insert(
                                var_name.to_string(),
                                VariableValue::Integer(energy_cost.0),
                            );
                    }
                    format!(
                        "{} gains {{reclaim-for-cost}} this turn.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
                (_, None) => {
                    format!(
                        "{} gains {{reclaim}} this turn.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::GainsSpark { target, gains } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "s",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            format!(
                "{} gains +{{s}} spark.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::EachMatchingGainsSpark { each, gains } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "s",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            format!(
                "have each {} gain +{{s}} spark.", serialize_allied_card_predicate(each,
                bindings)
            )
        }
        StandardEffect::EachMatchingGainsSparkForEach { each, for_each, .. } => {
            format!(
                "each {} gains spark equal to the number of {}.",
                serialize_allied_card_predicate(each, bindings),
                serialize_allied_card_predicate(for_each, bindings)
            )
        }
        StandardEffect::GainsSparkForQuantity { target, gains, for_quantity } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "s",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            if matches!(target, Predicate::This) {
                format!(
                    "gain +{{s}} spark for each {}.",
                    serialize_for_count_expression(for_quantity, bindings)
                )
            } else {
                format!(
                    "{} gains +{{s}} spark for each {}.",
                    predicate_serializer::serialize_predicate(target, bindings),
                    serialize_for_count_expression(for_quantity, bindings)
                )
            }
        }
        StandardEffect::SparkBecomes { matching, spark, .. } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "s",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(spark.0));
            }
            format!(
                "the spark of each {} becomes {{s}}.",
                serialize_allied_card_predicate(matching, bindings)
            )
        }
        StandardEffect::PutCardsFromYourDeckIntoVoid { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "top-n-cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "put the {top-n-cards} of your deck into your void.".to_string()
        }
        StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
            if *count == 1 {
                format!(
                    "put {} from your void on top of your deck.",
                    predicate_serializer::serialize_card_predicate(matching, bindings)
                )
            } else {
                format!(
                    "put {{up-to-n-cards}} {} from your void on top of your deck.",
                    predicate_serializer::serialize_card_predicate_plural(matching,
                    bindings)
                )
            }
        }
        StandardEffect::Counterspell { target } => {
            format!(
                "{{prevent}} a played {}.",
                predicate_serializer::predicate_base_text(target, bindings)
            )
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
                "gain control of {}.", predicate_serializer::serialize_predicate(target,
                bindings)
            )
        }
        StandardEffect::DissolveCharacter { target } => {
            format!(
                "{{dissolve}} {}.", predicate_serializer::serialize_predicate(target,
                bindings)
            )
        }
        StandardEffect::DissolveCharactersCount { target, count } => {
            match count {
                CollectionExpression::All => {
                    format!(
                        "{{dissolve}} all {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::Exactly(n) => {
                    format!(
                        "{{dissolve}} {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::UpTo(n) => {
                    format!(
                        "{{dissolve}} up to {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::AnyNumberOf => {
                    format!(
                        "{{dissolve}} any number of {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                _ => {
                    format!(
                        "{{dissolve}} {}.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::BanishCharacter { target } => {
            format!(
                "{{banish}} {}.", predicate_serializer::serialize_predicate(target,
                bindings)
            )
        }
        StandardEffect::BanishCollection { target, count } => {
            match count {
                CollectionExpression::AnyNumberOf => {
                    format!(
                        "{{banish}} any number of {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::All => {
                    format!(
                        "{{banish}} all {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::Exactly(n) => {
                    format!(
                        "{{banish}} {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                CollectionExpression::UpTo(n) => {
                    format!(
                        "{{banish}} up to {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                _ => {
                    format!(
                        "{{banish}} {}.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::BanishCardsFromEnemyVoid { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "{banish} {cards} from the opponent's void.".to_string()
        }
        StandardEffect::BanishEnemyVoid => "{banish} the opponent's void.".to_string(),
        StandardEffect::BanishThenMaterialize { target, count } => {
            match count {
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
                    if let Some(var_name) =
                        parser_substitutions::directive_to_integer_variable("up-to-n-allies")
                    {
                        bindings.insert(var_name.to_string(), VariableValue::Integer(*n));
                    }
                    if let Some(var_name) =
                        parser_substitutions::directive_to_integer_variable("it-or-them")
                    {
                        bindings.insert(var_name.to_string(), VariableValue::Integer(*n));
                    }
                    "{banish} {up-to-n-allies}, then {materialize} {it-or-them}.".to_string()
                }
                _ => {
                    format!(
                        "{{banish}} {}, then {{materialize}} them.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
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
                "{{materialize}} {}.", predicate_serializer::serialize_predicate(target,
                bindings)
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
                        "{{materialize}} {} copies of {}.", n,
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
                (_, QuantityExpression::Matching(predicate)) => {
                    format!(
                        "{{materialize}} a number of copies of {} equal to the number of {}.",
                        predicate_serializer::serialize_predicate(target, bindings),
                        predicate_serializer::serialize_predicate_plural(predicate,
                        bindings)
                    )
                }
                (_, QuantityExpression::ForEachEnergySpentOnThisCard) => {
                    format!(
                        "{{materialize}} a number of copies of {} equal to the amount of {{energy-symbol}} spent.",
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
            bindings.insert("figment".to_string(), VariableValue::Figment(*figment));
            if *count == 1 {
                "{materialize} {a-figment}.".to_string()
            } else {
                bindings.insert("number".to_string(), VariableValue::Integer(*count));
                "{materialize} {n-figments}.".to_string()
            }
        }
        StandardEffect::MaterializeFigmentsQuantity { count, quantity, figment } => {
            bindings.insert("figment".to_string(), VariableValue::Figment(*figment));
            let figment_text = if *count == 1 {
                "{a-figment}"
            } else {
                bindings.insert("number".to_string(), VariableValue::Integer(*count));
                "{n-figments}"
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
                        "{{materialize}} {} for each {}.", figment_text,
                        predicate_serializer::serialize_for_each_predicate(predicate,
                        bindings)
                    )
                }
                _ => {
                    format!(
                        "{{materialize}} {} for each {}.", figment_text,
                        serialize_for_count_expression(quantity, bindings)
                    )
                }
            }
        }
        StandardEffect::ReturnToHand { target } => {
            match target {
                Predicate::Any(CardPredicate::Character) => {
                    "return an enemy or ally to hand.".to_string()
                }
                Predicate::Another(CardPredicate::Character) => {
                    "return an ally to hand.".to_string()
                }
                Predicate::This => "return this character to your hand.".to_string(),
                _ => {
                    format!(
                        "return {} to hand.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
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
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "up-to-n-events",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "return {up-to-n-events} from your void to your hand.".to_string()
        }
        StandardEffect::ReturnFromYourVoidToPlay { target } => {
            format!(
                "{{reclaim}} {}.", predicate_serializer::serialize_predicate(target,
                bindings)
            )
        }
        StandardEffect::ReturnRandomFromYourVoidToPlay { predicate } => {
            format!(
                "{{reclaim}} a random {}.",
                predicate_serializer::serialize_card_predicate(predicate, bindings)
            )
        }
        StandardEffect::PutOnTopOfEnemyDeck { target } => {
            format!(
                "put {} on top of the opponent's deck.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::EachPlayerDiscardCards { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "discards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "each player discards {discards}.".to_string()
        }
        StandardEffect::EachPlayerAbandonsCharacters { matching, .. } => {
            format!(
                "each player abandons {}.",
                predicate_serializer::serialize_card_predicate(matching, bindings)
            )
        }
        StandardEffect::EachPlayerShufflesHandAndVoidIntoDeckAndDraws { count } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "cards",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            "each player shuffles their hand and void into their deck and then draws {cards}."
                .to_string()
        }
        StandardEffect::CardsInVoidGainReclaimThisTurn { count, predicate, until_end_of_turn } => {
            serialize_cards_in_void_gain_reclaim_this_turn(
                count,
                predicate,
                *until_end_of_turn,
                bindings,
            )
        }
        StandardEffect::MaterializeCollection { target, count } => {
            match (target, count) {
                (Predicate::Them, CollectionExpression::All) => {
                    "{materialize} them.".to_string()
                }
                (_, CollectionExpression::All) => {
                    format!(
                        "{{materialize}} all {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                (_, CollectionExpression::AnyNumberOf) => {
                    format!(
                        "{{materialize}} any number of {}.",
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                (_, CollectionExpression::UpTo(n)) => {
                    format!(
                        "{{materialize}} up to {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(target,
                        bindings)
                    )
                }
                _ => {
                    format!(
                        "{{materialize}} {}.",
                        predicate_serializer::serialize_predicate(target, bindings)
                    )
                }
            }
        }
        StandardEffect::MaterializeRandomFromDeck { count, predicate } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "n-random-characters",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
            }
            format!(
                "{{materialize}} {{n-random-characters}} {} from your deck.",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        StandardEffect::MultiplyYourEnergy { multiplier } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "multiplyby",
            ) {
                bindings
                    .insert(var_name.to_string(), VariableValue::Integer(*multiplier));
            }
            "{MultiplyBy} the amount of {energy-symbol} you have.".to_string()
        }
        StandardEffect::CopyNextPlayed { matching, times } => {
            if let Some(count) = times {
                if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                    "this-turn-times",
                ) {
                    bindings
                        .insert(var_name.to_string(), VariableValue::Integer(*count));
                }
            }
            format!(
                "copy the next {} you play {{this-turn-times}}.",
                predicate_serializer::predicate_base_text(matching, bindings)
            )
        }
        StandardEffect::Copy { target } => {
            format!(
                "copy {}.", predicate_serializer::serialize_predicate(target, bindings)
            )
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
        StandardEffect::TriggerJudgmentAbility { matching, collection } => {
            match collection {
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
                        "trigger the {{Judgment}} ability of {} {}.", n,
                        predicate_serializer::serialize_predicate_plural(matching,
                        bindings)
                    )
                }
                _ => {
                    format!(
                        "trigger the {{Judgment}} ability of {}.",
                        predicate_serializer::serialize_predicate(matching, bindings)
                    )
                }
            }
        }
        StandardEffect::TriggerAdditionalJudgmentPhaseAtEndOfTurn => {
            "at the end of this turn, trigger an additional {JudgmentPhaseName} phase."
                .to_string()
        }
        StandardEffect::TakeExtraTurn => "take an extra turn after this one.".to_string(),
        StandardEffect::YouWinTheGame => "you win the game.".to_string(),
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
            format!(
                "abandon {} and gain {{energy-symbol}} for each point of spark that character had.",
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
        StandardEffect::GainsAegisThisTurn { target } => {
            format!(
                "{} gains {{Aegis}} this turn.",
                predicate_serializer::serialize_predicate(target, bindings)
            )
        }
        StandardEffect::GainsSparkUntilYourNextMainForEach {
            target,
            gains,
            for_each,
        } => {
            if let Some(var_name) = parser_substitutions::directive_to_integer_variable(
                "s",
            ) {
                bindings.insert(var_name.to_string(), VariableValue::Integer(gains.0));
            }
            format!(
                "{} gains +{{s}} spark until your next main phase for each {}.",
                predicate_serializer::serialize_predicate(target, bindings),
                predicate_serializer::serialize_for_each_predicate(for_each, bindings)
            )
        }
        StandardEffect::GainTwiceThatMuchEnergyInstead => {
            "gain twice that much {energy-symbol} instead.".to_string()
        }
        StandardEffect::MaterializeCharacterFromVoid { target } => {
            format!(
                "{{materialize}} {} from your void.",
                predicate_serializer::serialize_card_predicate(target, bindings)
            )
        }
        StandardEffect::ThenMaterializeIt => "then {materialize} it.".to_string(),
        StandardEffect::NoEffect => "".to_string(),
        StandardEffect::OpponentPaysCost { cost } => {
            format!(
                "the opponent pays {}.", cost_serializer::serialize_cost(cost, bindings)
            )
        }
        StandardEffect::PayCost { cost } => {
            format!("pay {}.", cost_serializer::serialize_cost(cost, bindings))
        }
        StandardEffect::SpendAllEnergyDissolveEnemy => {
            "spend all your {energy-symbol}. {dissolve} an enemy with cost less than or equal to the amount spent."
                .to_string()
        }
        StandardEffect::SpendAllEnergyDrawAndDiscard => {
            "spend all your {energy-symbol}. Draw cards equal to the amount spent, then discard that many cards."
                .to_string()
        }
    }
}

pub fn serialize_effect(effect: &Effect, bindings: &mut VariableBindings) -> String {
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
                let effect_strings: Vec<String> = effects
                    .iter()
                    .map(|e| {
                        let s = serialize_standard_effect(&e.effect, bindings);
                        let s = s.trim_end_matches('.');
                        format!("{}.", serializer_utils::capitalize_first_letter(s))
                    })
                    .collect();
                result.push_str(&effect_strings.join(" "));
                result
            }
        }
        Effect::ListWithOptions(list_with_options) => {
            let mut result = String::new();
            if let Some(condition) = &list_with_options.condition {
                result.push_str(&condition_serializer::serialize_condition(condition, bindings));
                result.push(' ');
            }
            if let Some(trigger_cost) = &list_with_options.trigger_cost {
                result.push_str(&format!(
                    "{} to ",
                    cost_serializer::serialize_trigger_cost(trigger_cost, bindings)
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
                    serializer_utils::capitalize_first_letter(&serialize_effect(
                        &choice.effect,
                        bindings
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
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::Character) => {
            "ally abandoned this turn".to_string()
        }
        QuantityExpression::AbandonedThisTurn(CardPredicate::CharacterType(_)) => {
            "allied {subtype} abandoned this turn".to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::Character) => {
            "ally abandoned".to_string()
        }
        QuantityExpression::AbandonedThisWay(CardPredicate::CharacterType(_)) => {
            "allied {subtype} abandoned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::Character) => {
            "ally returned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(CardPredicate::CharacterType(_)) => {
            "allied {subtype} returned".to_string()
        }
        QuantityExpression::ReturnedToHandThisWay(predicate) => {
            format!(
                "{} returned",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::AbandonedThisTurn(predicate) => {
            format!(
                "{} abandoned this turn",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::AbandonedThisWay(predicate) => {
            format!(
                "{} abandoned",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::ForEachEnergySpentOnThisCard => "{energy-symbol} spent".to_string(),
        QuantityExpression::CardsDrawnThisTurn(predicate) => {
            format!(
                "{} you have drawn this turn",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::DiscardedThisTurn(predicate) => {
            format!(
                "{} you have discarded this turn",
                text_formatting::card_predicate_base_text(predicate).without_article()
            )
        }
        QuantityExpression::DissolvedThisTurn(predicate) => {
            format!(
                "{} which dissolved this turn",
                text_formatting::card_predicate_base_text(predicate).without_article()
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
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            "allied {subtype}".to_string()
        }
        _ => {
            format!(
                "allied {}",
                text_formatting::card_predicate_base_text(card_predicate).without_article()
            )
        }
    }
}

fn serialize_cards_in_void_gain_reclaim_this_turn(
    count: &CollectionExpression,
    predicate: &CardPredicate,
    until_end_of_turn: bool,
    bindings: &mut VariableBindings,
) -> String {
    let this_turn_suffix = if until_end_of_turn { " this turn" } else { "" };
    match count {
        CollectionExpression::Exactly(1) => {
            let predicate_text = if let CardPredicate::CharacterType(subtype) = predicate {
                bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
                "{ASubtype}".to_string()
            } else {
                text_formatting::card_predicate_base_text(predicate).capitalized_with_article()
            };
            format!(
                "{} in your void gains {{reclaim}} equal to its cost{}.",
                predicate_text, this_turn_suffix
            )
        }
        CollectionExpression::Exactly(n) => {
            format!(
                "{} {} in your void gain {{reclaim}} equal to their cost{}.",
                n,
                predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
                this_turn_suffix
            )
        }
        CollectionExpression::All => {
            format!(
                "all cards currently in your void gain {{reclaim}} equal to their cost{}.",
                this_turn_suffix
            )
        }
        CollectionExpression::AllButOne => {
            format!(
                "all but one {} in your void gain {{reclaim}} equal to their cost{}.",
                predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
                this_turn_suffix
            )
        }
        CollectionExpression::UpTo(n) => {
            format!(
                "up to {} {} in your void gain {{reclaim}} equal to their cost{}.",
                n,
                predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
                this_turn_suffix
            )
        }
        CollectionExpression::AnyNumberOf => {
            format!(
                "any number of {} in your void gain {{reclaim}} equal to their cost{}.",
                predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
                this_turn_suffix
            )
        }
        CollectionExpression::OrMore(n) => {
            format!(
                "{} or more {} in your void gain {{reclaim}} equal to their cost{}.",
                n,
                predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
                this_turn_suffix
            )
        }
        CollectionExpression::EachOther => {
            format!(
                "Each other card in your void gains {{reclaim}} equal to its cost{}",
                this_turn_suffix
            )
        }
    }
}
