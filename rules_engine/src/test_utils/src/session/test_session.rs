use std::thread;
use std::time::Duration;

use action_data::game_action_data::GameAction;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use core_data::identifiers::{BattleId, UserId};
use display_data::request_data::{
    ConnectRequest, Metadata, PerformActionRequest, PollResponseType,
};
use rules_engine::engine;
use tokio::time;

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
    /// on the tokio blocking thread. It will then wait a response to be
    /// available in the poll results queue marked with PollResponseType::Final.
    /// This means that once this function completes, the action has been
    /// fully executed.
    pub async fn perform_action(&mut self, action: impl Into<GameAction>) {
        let action = action.into();

        let request = PerformActionRequest {
            metadata: self.metadata(),
            action,
            vs_opponent: None,
            test_scenario: None,
        };

        engine::perform_action(self.state_provider.clone(), request);

        loop {
            time::sleep(Duration::from_millis(10)).await;

            if let Some(response) = engine::poll(self.state_provider.clone(), self.user_id) {
                if let Some(battle_id) = response.metadata.battle_id {
                    self.battle_id = Some(battle_id);
                }

                if let Some(commands) = response.commands {
                    self.client.apply_commands(commands);
                }

                if matches!(response.response_type, PollResponseType::Final) {
                    break;
                }
            }
        }
    }

    pub fn perform_action_blocking(&mut self, action: impl Into<GameAction>) {
        let action = action.into();

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();

        let request = PerformActionRequest {
            metadata: self.metadata(),
            action,
            vs_opponent: None,
            test_scenario: None,
        };

        engine::perform_action(self.state_provider.clone(), request);

        loop {
            thread::sleep(Duration::from_millis(10));

            if let Some(response) = engine::poll(self.state_provider.clone(), self.user_id) {
                if let Some(battle_id) = response.metadata.battle_id {
                    self.battle_id = Some(battle_id);
                }

                if let Some(commands) = response.commands {
                    self.client.apply_commands(commands);
                }

                if matches!(response.response_type, PollResponseType::Final) {
                    break;
                }
            }
        }
    }

    fn metadata(&self) -> Metadata {
        Metadata { user_id: self.user_id, battle_id: self.battle_id, request_id: None }
    }

    fn request_context(&self) -> RequestContext {
        RequestContext { logging_options: LoggingOptions::default() }
    }
}
