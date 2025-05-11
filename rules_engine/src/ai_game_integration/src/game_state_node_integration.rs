use ai_core::game_state_node::{GameStateNode, GameStatus};
use battle_mutations::actions::apply_battle_action;
use battle_mutations::card_mutations::player_hand;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use core_data::types::PlayerName;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use tracing_macros::panic_with;

/// Wrapper over [BattleState] to allow trait to be implemented in this crate.
pub struct AgentBattleState {
    pub state: BattleState,
}

impl GameStateNode for AgentBattleState {
    type Action = BattleAction;
    type PlayerName = PlayerName;

    fn make_copy(&self) -> Self {
        Self { state: self.state.logical_clone() }
    }

    fn make_randomized_copy(&self, player: PlayerName) -> Self
    where
        Self: Sized,
    {
        let mut result = self.state.logical_clone();
        let seed = rand::rng().random();
        result.rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        result.seed = seed;
        player_hand::randomize_player_hand(&mut result, player);
        Self { state: result }
    }

    fn status(&self) -> GameStatus<PlayerName> {
        match self.state.status {
            BattleStatus::GameOver { winner } => GameStatus::Completed { winner },
            _ => {
                let Some(next) = legal_actions::next_to_act(&self.state) else {
                    panic_with!("No player to act", &self.state);
                };
                GameStatus::InProgress { current_turn: next }
            }
        }
    }

    fn legal_actions<'a>(
        &'a self,
        player: PlayerName,
    ) -> Box<dyn Iterator<Item = BattleAction> + 'a> {
        let actions = legal_actions::compute(&self.state, player).all();
        Box::new(actions.into_iter())
    }

    fn execute_action(&mut self, player: PlayerName, action: BattleAction) {
        apply_battle_action::execute(&mut self.state, player, action);
    }
}
