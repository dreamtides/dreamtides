use ability_data::predicate::{CardPredicate, Operator, Predicate};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType, CharacterId, VoidCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::card_types::CardType;
use core_data::numerics::{Energy, Spark};

use crate::battle_card_queries::card_properties;

/// Returns true if the given [Predicate] currently matches for the given card.
pub fn matches(
    battle: &BattleState,
    source: EffectSource,
    predicate: &Predicate,
    card_id: impl CardIdType,
) -> bool {
    let card_id = card_id.card_id();
    let controller = source.controller();

    match predicate {
        Predicate::This => source.card_id() == Some(card_id),
        Predicate::It => true,
        Predicate::Them => true,
        Predicate::That => true,
        Predicate::Enemy(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller.opponent() {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Another(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller {
                return false;
            }
            if source.card_id() == Some(card_id) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Your(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::Any(card_predicate) => {
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::AnyOther(card_predicate) => {
            if source.card_id() == Some(card_id) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::YourVoid(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller {
                return false;
            }
            if !battle.cards.void(controller).contains(VoidCardId(card_id)) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
        Predicate::EnemyVoid(card_predicate) => {
            let card_controller = card_properties::controller(battle, card_id);
            if card_controller != controller.opponent() {
                return false;
            }
            if !battle.cards.void(controller.opponent()).contains(VoidCardId(card_id)) {
                return false;
            }
            matches_card_predicate(battle, source, card_predicate, card_id)
        }
    }
}

fn matches_card_predicate(
    battle: &BattleState,
    source: EffectSource,
    predicate: &CardPredicate,
    card_id: CardId,
) -> bool {
    match predicate {
        CardPredicate::Card => true,
        CardPredicate::Character => {
            matches!(card_properties::card_type(battle, card_id), CardType::Character(_))
        }
        CardPredicate::Event => card_properties::card_type(battle, card_id) == CardType::Event,
        CardPredicate::Dream => true,
        CardPredicate::CharacterType(character_type) => {
            match card_properties::card_type(battle, card_id) {
                CardType::Character(card_character_type) => card_character_type == *character_type,
                _ => false,
            }
        }
        CardPredicate::NotCharacterType(character_type) => {
            match card_properties::card_type(battle, card_id) {
                CardType::Character(card_character_type) => card_character_type != *character_type,
                _ => false,
            }
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            let controller = card_properties::controller(battle, card_id);
            let Some(card_spark) = battle.cards.spark(controller, CharacterId(card_id)) else {
                return false;
            };
            match operator {
                Operator::LowerBy(amount) => card_spark == Spark(spark.0 - amount.0),
                Operator::OrLess => card_spark <= *spark,
                Operator::Exactly => card_spark == *spark,
                Operator::OrMore => card_spark >= *spark,
                Operator::HigherBy(amount) => card_spark == Spark(spark.0 + amount.0),
            }
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            let Some(card_cost) = card_properties::cost(battle, card_id) else {
                return false;
            };
            match cost_operator {
                Operator::LowerBy(amount) => card_cost == Energy(cost.0 - amount.0),
                Operator::OrLess => card_cost <= *cost,
                Operator::Exactly => card_cost == *cost,
                Operator::OrMore => card_cost >= *cost,
                Operator::HigherBy(amount) => card_cost == Energy(cost.0 + amount.0),
            }
        }
        CardPredicate::CharacterWithCostComparedToControlled {
            target,
            cost_operator,
            count_matching,
        } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            let Some(card_cost) = card_properties::cost(battle, card_id) else {
                return false;
            };
            let controller = source.controller();
            let controlled_count = battle
                .cards
                .battlefield(controller)
                .iter()
                .filter(|character_id| {
                    matches_card_predicate(battle, source, count_matching, character_id.card_id())
                })
                .count();
            let comparison_cost = Energy(controlled_count as u32);
            match cost_operator {
                Operator::LowerBy(amount) => {
                    card_cost == Energy(comparison_cost.0.saturating_sub(amount.0))
                }
                Operator::OrLess => card_cost <= comparison_cost,
                Operator::Exactly => card_cost == comparison_cost,
                Operator::OrMore => card_cost >= comparison_cost,
                Operator::HigherBy(amount) => card_cost == Energy(comparison_cost.0 + amount.0),
            }
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, cost_operator } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            let Some(card_cost) = card_properties::cost(battle, card_id) else {
                return false;
            };
            let controller = source.controller();
            let abandoned_count = battle
                .turn_history
                .current_action_history
                .player(controller)
                .character_limit_characters_abandoned
                .len();
            let comparison_cost = Energy(abandoned_count as u32);
            match cost_operator {
                Operator::LowerBy(amount) => {
                    card_cost == Energy(comparison_cost.0.saturating_sub(amount.0))
                }
                Operator::OrLess => card_cost <= comparison_cost,
                Operator::Exactly => card_cost == comparison_cost,
                Operator::OrMore => card_cost >= comparison_cost,
                Operator::HigherBy(amount) => card_cost == Energy(comparison_cost.0 + amount.0),
            }
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, spark_operator } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            let controller_of_card = card_properties::controller(battle, card_id);
            let Some(card_spark) = battle.cards.spark(controller_of_card, CharacterId(card_id))
            else {
                return false;
            };
            let controller = source.controller();
            let abandoned_count = battle
                .turn_history
                .current_action_history
                .player(controller)
                .character_limit_characters_abandoned
                .len();
            let comparison_spark = Spark(abandoned_count as u32);
            match spark_operator {
                Operator::LowerBy(amount) => {
                    card_spark == Spark(comparison_spark.0.saturating_sub(amount.0))
                }
                Operator::OrLess => card_spark <= comparison_spark,
                Operator::Exactly => card_spark == comparison_spark,
                Operator::OrMore => card_spark >= comparison_spark,
                Operator::HigherBy(amount) => card_spark == Spark(comparison_spark.0 + amount.0),
            }
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn {
            target,
            spark_operator,
        } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            let controller_of_card = card_properties::controller(battle, card_id);
            let Some(card_spark) = battle.cards.spark(controller_of_card, CharacterId(card_id))
            else {
                return false;
            };
            let controller = source.controller();
            let abandoned_count = battle
                .turn_history
                .current_action_history
                .player(controller)
                .character_limit_characters_abandoned
                .len();
            let comparison_spark = Spark(abandoned_count as u32);
            match spark_operator {
                Operator::LowerBy(amount) => {
                    card_spark == Spark(comparison_spark.0.saturating_sub(amount.0))
                }
                Operator::OrLess => card_spark <= comparison_spark,
                Operator::Exactly => card_spark == comparison_spark,
                Operator::OrMore => card_spark >= comparison_spark,
                Operator::HigherBy(amount) => card_spark == Spark(comparison_spark.0 + amount.0),
            }
        }
        CardPredicate::CharacterWithMaterializedAbility => false,
        CardPredicate::Fast { target } => {
            if !matches_card_predicate(battle, source, target, card_id) {
                return false;
            }
            card_properties::is_fast(battle, card_id)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => false,
    }
}
