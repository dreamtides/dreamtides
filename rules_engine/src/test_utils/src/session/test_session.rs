use action_data::game_action_data::GameAction;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use display_data::request_data::{
    ConnectRequest, DebugConfiguration, Metadata, PerformActionRequest,
};
use rules_engine::engine;

use crate::client::test_client::TestClient;
use crate::provider::test_state_provider::TestStateProvider;

pub struct TestSession {
    pub state_provider: TestStateProvider,
    pub user_id: UserId,
    pub enemy_id: UserId,
    pub battle_id: Option<BattleId>,
    pub client: TestClient,
    pub enemy_client: TestClient,
}

impl TestSession {
    /// Connects to the rules engine and applies the commands to the client.
    pub fn connect(&mut self) {
        let response = engine::connect(
            self.state_provider.clone(),
            &ConnectRequest {
                metadata: self.metadata(),
                persistent_data_path: "".to_string(),
                vs_opponent: None,
                test_scenario: None,
                display_properties: None,
                debug_configuration: Some(DebugConfiguration {
                    enemy: Some(PlayerType::User(self.enemy_id)),
                    seed: Some(314159265358979323),
                }),
            },
            self.request_context(),
        );

        if let Some(battle_id) = response.metadata.battle_id {
            self.battle_id = Some(battle_id);
        }

        self.client.apply_commands(response.commands);

        let enemy_response = engine::connect(
            self.state_provider.clone(),
            &ConnectRequest {
                metadata: self.enemy_metadata(),
                persistent_data_path: "".to_string(),
                vs_opponent: Some(self.user_id),
                test_scenario: None,
                display_properties: None,
                debug_configuration: None,
            },
            self.request_context(),
        );

        self.enemy_client.apply_commands(enemy_response.commands);
    }

    /// Performs a Game Action.
    ///
    /// This function will call the rules engine to execute the provided action
    /// synchronously. It blocks until the action has been fully executed and
    /// returns after applying all commands to the client.
    pub fn perform_action(&mut self, action: impl Into<GameAction>) {
        self.perform_action_as_player(action, self.metadata(), self.user_id, self.enemy_id);
    }

    /// Performs a Game Action as the enemy player.
    pub fn perform_enemy_action(&mut self, action: impl Into<GameAction>) {
        self.perform_action_as_player(action, self.enemy_metadata(), self.user_id, self.user_id);
    }

    fn perform_action_as_player(
        &mut self,
        action: impl Into<GameAction>,
        metadata: Metadata,
        save_file_id: UserId,
        opponent_id: UserId,
    ) {
        let action = action.into();

        let request = PerformActionRequest {
            metadata,
            action,
            save_file_id: Some(save_file_id),
            test_scenario: None,
        };

        let result = engine::perform_action_blocking(
            self.state_provider.clone(),
            request,
            Some(opponent_id),
        );

        for poll_result in result.user_poll_results {
            if metadata.user_id == self.user_id {
                self.client.apply_commands(poll_result.commands);
            } else {
                self.enemy_client.apply_commands(poll_result.commands);
            }
        }

        for poll_result in result.enemy_poll_results {
            if opponent_id == self.user_id {
                self.client.apply_commands(poll_result.commands);
            } else {
                self.enemy_client.apply_commands(poll_result.commands);
            }
        }
    }

    fn metadata(&self) -> Metadata {
        Metadata { user_id: self.user_id, battle_id: self.battle_id, request_id: None }
    }

    fn enemy_metadata(&self) -> Metadata {
        Metadata { user_id: self.enemy_id, battle_id: self.battle_id, request_id: None }
    }

    fn request_context(&self) -> RequestContext {
        RequestContext { logging_options: LoggingOptions::default() }
    }
}
