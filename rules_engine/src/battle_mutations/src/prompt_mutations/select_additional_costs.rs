use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardAdditionalCostsPaid;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use tracing_macros::{battle_trace, panic_with};

use crate::player_mutations::energy;

pub fn energy_cost(battle: &mut BattleState, player: PlayerName, cost: Energy) {
    let Some(source) = battle.prompt.as_ref().map(|p| p.source) else {
        panic_with!("No active prompt for applying additional cost", battle, cost);
    };

    let Some(stack_card) = battle.cards.top_of_stack_mut() else {
        panic_with!("No active stack for applying additional cost", battle, cost);
    };

    stack_card.additional_costs_paid = StackCardAdditionalCostsPaid::Energy(cost);
    battle_trace!("Paying additional cost", battle, player, cost);
    energy::spend(battle, player, source, cost);
    battle.prompt = None;
}
