use ability_data::ability::Ability;
use ability_data::cost::Cost;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::{CardIdType, StackCardId};
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;

/// Returns true if the given card has legal choices for its additional costs
pub fn for_event(battle: &BattleData, card_id: impl CardIdType, paid: Energy) -> bool {
    let Some(card) = battle.cards.card(card_id) else {
        return false;
    };

    let controller = card.controller();
    for (ability_index, ability) in card.abilities.iter().enumerate() {
        if let Ability::Event(event) = ability
            && let Some(additional_cost) = event.additional_cost.as_ref()
        {
            let source = EffectSource::Event {
                controller,
                stack_card_id: StackCardId(card_id.card_id()),
                ability_number: AbilityNumber(ability_index),
            };
            if !has_legal_additional_cost_choices(battle, source, additional_cost, paid) {
                return false;
            }
        }
    }

    true
}

fn has_legal_additional_cost_choices(
    battle: &BattleData,
    source: EffectSource,
    cost: &Cost,
    paid: Energy,
) -> bool {
    match cost {
        Cost::SpendOneOrMoreEnergy => battle.player(source.controller()).current_energy > paid,
        _ => todo!("Implement additional cost choices"),
    }
}
