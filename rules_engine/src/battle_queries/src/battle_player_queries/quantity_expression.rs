use ability_data::quantity_expression_data::QuantityExpression;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardAdditionalCostsPaid;
use battle_state::core::effect_source::EffectSource;

/// Returns the number of items which currently match this [QuantityExpression].
///
/// Returns 0 in cases where the expression doesn't match, e.g. checking the
/// additional costs paid for a card which is no longer present on the stack.
pub fn count(battle: &BattleState, source: EffectSource, expression: &QuantityExpression) -> u32 {
    match expression {
        QuantityExpression::ForEachEnergySpentOnThisCard => {
            if let EffectSource::Event { stack_card_id, .. } = source
                && let Some(card) = battle.cards.stack_card(stack_card_id)
                && let StackCardAdditionalCostsPaid::Energy(energy) = card.additional_costs_paid
            {
                energy.0
            } else {
                0
            }
        }
        _ => todo!("Implement {:?}", expression),
    }
}
