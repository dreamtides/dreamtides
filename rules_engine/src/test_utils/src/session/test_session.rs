use action_data::game_action_data::GameAction;
use ai_data::game_ai::GameAI;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
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
    pub battle_id: Option<BattleId>,
    pub client: TestClient,
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
                    enemy_agent: Some(GameAI::FirstAvailableAction),
                    seed: Some(314159265358979323),
                }),
            },
            self.request_context(),
        );

        if let Some(battle_id) = response.metadata.battle_id {
            self.battle_id = Some(battle_id);
        }

        self.client.apply_commands(response.commands);
    }

    /// Performs a Game Action.
    ///
    /// This function will call the rules engine to execute the provided action
    /// synchronously. It blocks until the action has been fully executed and
    /// returns after applying all commands to the client.
    pub fn perform_action(&mut self, action: impl Into<GameAction>) {
        let action = action.into();

        let request = PerformActionRequest {
            metadata: self.metadata(),
            action,
            vs_opponent: None,
            test_scenario: None,
        };

        let poll_results = engine::perform_action_blocking(self.state_provider.clone(), request);
        for poll_result in poll_results {
            self.client.apply_commands(poll_result.commands);
        }
    }

    fn metadata(&self) -> Metadata {
        Metadata { user_id: self.user_id, battle_id: self.battle_id, request_id: None }
    }

    fn request_context(&self) -> RequestContext {
        RequestContext { logging_options: LoggingOptions::default() }
    }
}
