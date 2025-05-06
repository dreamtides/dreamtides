use assert_with::expect;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::additional_cost_choice_data::AdditionalCostData;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::player_mutations::energy;

pub fn energy_cost(battle: &mut BattleData, player: PlayerName, cost: Energy) {
    if battle.cards.top_of_stack_mut().is_none() {
        return;
    }

    let source = expect!(battle.prompt.as_ref().map(|p| p.source), battle, || format!(
        "No active prompt for applying additional cost {:?}",
        cost
    ));
    let stack_card = expect!(battle.cards.top_of_stack_mut(), battle, || format!(
        "No active stack for applying additional cost {:?}",
        cost
    ));
    stack_card.additional_cost_choices.push(AdditionalCostData::PayEnergy(cost));
    energy::spend(battle, player, source, cost);
    battle.prompt = None;
}
