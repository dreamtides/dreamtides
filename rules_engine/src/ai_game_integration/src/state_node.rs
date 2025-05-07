use std::ops::{Deref, DerefMut};

use actions::battle_actions;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use assert_with::{assert_that, expect};
use battle_data_old::actions::battle_action_data::BattleAction;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::battle_status::BattleStatus;
use battle_queries_old::legal_action_queries::legal_actions::{self, LegalActions};
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
        Self(self.clone_for_ai_search())
    }

    fn make_randomized_copy(&self, player: PlayerName) -> Self
    where
        Self: Sized,
    {
        let mut result = self.clone_for_ai_search();
        result.cards.randomize_player(player.opponent(), &mut result.rng);
        Self(result)
    }

    fn status(&self) -> GameStatus<PlayerName> {
        match self.status {
            BattleStatus::GameOver { winner } => GameStatus::Completed { winner },
            _ => GameStatus::InProgress {
                current_turn: expect!(legal_actions::next_to_act(self), self, || {
                    "No player to act"
                }),
            },
        }
    }

    fn legal_actions<'a>(
        &'a self,
        player: PlayerName,
    ) -> Box<dyn Iterator<Item = BattleAction> + 'a> {
        let actions =
            legal_actions::compute(self, player, LegalActions { for_human_player: false });
        assert_that!(!actions.is_empty(), self, || format!(
            "No legal actions for player: {:?}",
            player
        ));
        Box::new(actions.into_iter())
    }

    fn execute_action(&mut self, player: PlayerName, action: BattleAction) {
        battle_actions::execute(self, player, action);
    }
}
