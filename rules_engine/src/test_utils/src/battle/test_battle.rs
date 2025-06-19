use battle_state::actions::battle_actions::{BattleAction, DebugBattleAction};
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use uuid::Uuid;

use crate::battle::test_player::TestPlayer;
use crate::client::test_client::TestClient;
use crate::provider::test_state_provider::TestStateProvider;
use crate::session::test_session::TestSession;

pub struct TestBattle {
    pub session: TestSession,
    pub user: TestPlayer,
    pub opponent: TestPlayer,
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
            user: TestPlayer::default(),
            opponent: TestPlayer::default(),
        }
    }

    /// Connects to the rules engine, returning the session struct.
    ///
    /// Applies debug commands to populate the current battle state.
    pub fn connect(mut self) -> TestSession {
        self.session.connect();
        self.apply_test_player_configuration();
        self.session
    }

    fn apply_test_player_configuration(&mut self) {
        let user_config = self.user.clone();
        let opponent_config = self.opponent.clone();
        self.apply_player_configuration(PlayerName::One, &user_config);
        self.apply_player_configuration(PlayerName::Two, &opponent_config);
    }

    fn apply_player_configuration(&mut self, player: PlayerName, config: &TestPlayer) {
        if let Some(points) = config.points {
            self.session
                .perform_action(BattleAction::Debug(DebugBattleAction::SetPoints(player, points)));
        }
        if let Some(energy) = config.energy {
            self.session
                .perform_action(BattleAction::Debug(DebugBattleAction::SetEnergy(player, energy)));
        }
        if let Some(produced_energy) = config.produced_energy {
            self.session.perform_action(BattleAction::Debug(DebugBattleAction::SetProducedEnergy(
                player,
                produced_energy,
            )));
        }
        if let Some(spark_bonus) = config.spark_bonus {
            self.session.perform_action(BattleAction::Debug(DebugBattleAction::SetSparkBonus(
                player,
                spark_bonus,
            )));
        }
        for card_name in &config.hand {
            self.session.perform_action(BattleAction::Debug(DebugBattleAction::AddCardToHand(
                player, *card_name,
            )));
        }
        for card_name in &config.battlefield {
            self.session.perform_action(BattleAction::Debug(
                DebugBattleAction::AddCardToBattlefield(player, *card_name),
            ));
        }
        for card_name in &config.void {
            self.session.perform_action(BattleAction::Debug(DebugBattleAction::AddCardToVoid(
                player, *card_name,
            )));
        }
    }
}
