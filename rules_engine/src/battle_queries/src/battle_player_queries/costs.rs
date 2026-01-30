use ability_data::cost::Cost;
use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptChoiceLabel;
use core_data::types::PlayerName;
use tabula_generated::string_id::StringId;

/// Returns true if the [PlayerName] player can pay a [Cost].
pub fn can_pay(battle: &BattleState, player: PlayerName, cost: &Cost) -> bool {
    match cost {
        Cost::Energy(energy) => battle.players.player(player).current_energy >= *energy,
        _ => todo!("Implement {:?}", cost),
    }
}

/// Returns a [PromptChoiceLabel] for choosing to pay a [Cost].
pub fn pay_cost_label(cost: &Cost) -> PromptChoiceLabel {
    match cost {
        Cost::Energy(energy) => {
            PromptChoiceLabel::StringWithEnergy(StringId::PayEnergyPromptButton, *energy)
        }
        _ => todo!("Implement {:?}", cost),
    }
}
