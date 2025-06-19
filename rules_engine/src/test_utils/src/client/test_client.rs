use crate::client::test_client_cards::TestClientCards;
use crate::client::test_client_player::TestClientPlayer;

/// Represents a user client connected to a test game
#[derive(Default)]
pub struct TestClient {
    pub cards: TestClientCards,
    /// A player's view of *their own* player state.
    pub user: TestClientPlayer,
    /// A player's view of *their opponent's* player state.
    pub enemy: TestClientPlayer,
}

impl TestClient {}
