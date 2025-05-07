use ability_data::quantity_expression_data::QuantityExpression;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle_cards::additional_cost_choice_data::AdditionalCostData;

/// Returns the number of items which currently match this [QuantityExpression].
///
/// Returns 0 in cases where the expression doesn't match, e.g. checking the
/// additional costs paid for a card which is no longer present on the stack.
pub fn count(battle: &BattleData, source: EffectSource, expression: QuantityExpression) -> u32 {
    match expression {
        QuantityExpression::ForEachEnergySpentOnThisCard => {
            let EffectSource::Event { stack_card_id, .. } = source else {
                return 0;
            };
            let Some(card) = battle.cards.card(stack_card_id) else {
                return 0;
            };
            let sum = card
                .additional_cost_choices
                .iter()
                .map(|cost| match cost {
                    AdditionalCostData::PayEnergy(energy) => energy.0,
                })
                .sum();
            sum
        }
        _ => todo!("Implement {:?}", expression),
    }
}
