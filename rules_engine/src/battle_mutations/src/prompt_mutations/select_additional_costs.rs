use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackCardAdditionalCostsPaid;
use battle_state::prompt_types::prompt_data::PromptChoiceLabel;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

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
    battle.push_animation(source, || BattleAnimation::MakeChoice {
        player,
        choice: PromptChoiceLabel::PayEnergy(cost),
    });
    energy::spend(battle, player, source, cost);
    battle.prompt = None;
}
