use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;

pub fn select_action(battle: &BattleData, player: PlayerName) -> BattleAction {
    *legal_actions::compute(battle, player, LegalActions { for_human_player: false })
        .first()
        .expect("Invoked agent search with no legal actions available")
}
