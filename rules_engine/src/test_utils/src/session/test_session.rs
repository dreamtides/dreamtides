use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use core_data::identifiers::{BattleId, UserId};
use display_data::request_data::{ConnectRequest, Metadata};
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
            },
            self.request_context(),
        );

        if let Some(battle_id) = response.metadata.battle_id {
            self.battle_id = Some(battle_id);
        }

        self.client.apply_commands(response.commands);
    }

    fn metadata(&self) -> Metadata {
        Metadata { user_id: self.user_id, battle_id: self.battle_id, request_id: None }
    }

    fn request_context(&self) -> RequestContext {
        RequestContext { logging_options: LoggingOptions::default() }
    }
}
