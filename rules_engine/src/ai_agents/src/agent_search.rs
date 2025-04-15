use action_data::battle_action::BattleAction;
use ai_data::agent::Agent;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;
use rand::seq::IndexedRandom;

pub fn select_action(battle: &BattleData, player: PlayerName, agent: &Agent) -> BattleAction {
    match agent {
        Agent::FirstAvailableAction => {
            *legal_actions::compute(battle, player, LegalActions { for_human_player: false })
                .first()
                .expect("Invoked agent search with no legal actions available")
        }
        Agent::RandomAction => {
            *legal_actions::compute(battle, player, LegalActions { for_human_player: false })
                .choose(&mut rand::rng())
                .expect("Invoked agent search with no legal actions available")
        }
    }
}
