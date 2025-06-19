use core_data::identifiers::{BattleId, UserId};
use uuid::Uuid;

use crate::client::test_client::TestClient;
use crate::provider::test_state_provider::TestStateProvider;
use crate::session::test_session::TestSession;

pub struct TestBattle {
    pub session: TestSession,
}

impl Default for TestBattle {
    fn default() -> Self {
        Self::new()
    }
}

impl TestBattle {
    /// Creates a new battle with a random user ID and battle ID, playing as
    /// player one.
    pub fn new() -> Self {
        Self {
            session: TestSession {
                state_provider: TestStateProvider::default(),
                user_id: UserId(Uuid::new_v4()),
                battle_id: Some(BattleId(Uuid::new_v4())),
                client: TestClient::default(),
            },
        }
    }

    /// Connects to the rules engine, returning the session struct.
    pub fn connect(mut self) -> TestSession {
        self.session.connect();
        self.session
    }
}
