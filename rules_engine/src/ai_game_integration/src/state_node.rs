use std::ops::{Deref, DerefMut};

use action_data::battle_action::BattleAction;
use actions::battle_actions;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;

/// Wrapper over [BattleData] to allow trait to be implemented in this crate.
pub struct AgentBattleState(pub BattleData);

impl Deref for AgentBattleState {
    type Target = BattleData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AgentBattleState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl GameStateNode for AgentBattleState {
    type Action = BattleAction;
    type PlayerName = PlayerName;

    fn make_copy(&self) -> Self {
        Self(self.0.clone())
    }

    fn status(&self) -> GameStatus<PlayerName> {
        match self.status {
            BattleStatus::GameOver { winner } => GameStatus::Completed { winner },
            _ => GameStatus::InProgress { current_turn: legal_actions::next_to_act(self).unwrap() },
        }
    }

    fn legal_actions<'a>(
        &'a self,
        player: PlayerName,
    ) -> Box<dyn Iterator<Item = BattleAction> + 'a> {
        Box::new(
            legal_actions::compute(self, player, LegalActions { for_human_player: false })
                .into_iter(),
        )
    }

    fn execute_action(&mut self, player: PlayerName, action: BattleAction) {
        battle_actions::execute(self, player, action);
    }
}
