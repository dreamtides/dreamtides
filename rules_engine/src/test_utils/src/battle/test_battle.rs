use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use uuid::Uuid;

use crate::battle::test_player::TestPlayer;
use crate::session::test_session::TestSession;

pub struct TestBattle {
    pub session: TestSession,
    pub user: TestPlayer,
    pub enemy: TestPlayer,
}

impl Default for TestBattle {
    fn default() -> Self {
        Self::builder()
    }
}

impl TestBattle {
    /// Creates a new battle with a random user ID and battle ID, playing as
    /// player one.
    pub fn builder() -> Self {
        let mut session = TestSession::new();
        session.battle_id = Some(BattleId(Uuid::new_v4()));
        Self { session, user: TestPlayer::default(), enemy: TestPlayer::default() }
    }

    /// Sets the user player to the provided player.
    pub fn user(mut self, user: TestPlayer) -> Self {
        self.user = user;
        self
    }

    /// Sets the enemy player to the provided player.
    pub fn enemy(mut self, enemy: TestPlayer) -> Self {
        self.enemy = enemy;
        self
    }

    /// Connects to the rules engine, returning the session struct.
    ///
    /// Applies debug commands to populate the current battle state.
    pub fn connect(mut self) -> TestSession {
        self.session.connect();
        self.move_all_hands_to_deck();
        self.apply_test_player_configuration();
        self.session
    }

    fn apply_test_player_configuration(&mut self) {
        let user_config = self.user.clone();
        let opponent_config = self.enemy.clone();
        self.apply_player_configuration(PlayerName::One, &user_config);
        self.apply_player_configuration(PlayerName::Two, &opponent_config);
    }

    fn move_all_hands_to_deck(&mut self) {
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::MoveHandToDeck(
            PlayerName::One,
        )));
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::MoveHandToDeck(
            PlayerName::Two,
        )));
    }

    fn apply_player_configuration(&mut self, player: PlayerName, config: &TestPlayer) {
        if let Some(points) = config.points {
            self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::SetPoints(
                player, points,
            )));
        }
        if let Some(energy) = config.energy {
            self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::SetEnergy(
                player, energy,
            )));
        }
        if let Some(produced_energy) = config.produced_energy {
            self.session.perform_user_action(BattleAction::Debug(
                DebugBattleAction::SetProducedEnergy(player, produced_energy),
            ));
        }
        if let Some(spark_bonus) = config.spark_bonus {
            self.session.perform_user_action(BattleAction::Debug(
                DebugBattleAction::SetSparkBonus(player, spark_bonus),
            ));
        }
    }
}
