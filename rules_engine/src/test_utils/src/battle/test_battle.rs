use ai_data::game_ai::GameAI;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use tabula_ids::card_lists::DreamwellCardIdList;
use uuid::Uuid;

use crate::battle::test_player::TestPlayer;
use crate::session::test_session::TestSession;

pub struct TestBattle {
    pub session: TestSession,
    pub user: TestPlayer,
    pub enemy: TestPlayer,
    pub enemy_agent: Option<GameAI>,
}

impl Default for TestBattle {
    fn default() -> Self {
        Self::builder()
    }
}

impl TestBattle {
    /// Creates a new battle with a random user ID and battle ID, playing as
    /// player one.
    ///
    /// By default, this will create a battle state where:
    ///
    ///   - It is the user's turn
    ///   - Both players have 99 energy
    ///   - Both players have 99 produced energy
    ///   - Neither player has cards in hand
    pub fn builder() -> Self {
        let mut session = TestSession::new();
        session.battle_id = Some(BattleId(Uuid::new_v4()));
        Self {
            session,
            user: TestPlayer::default(),
            enemy: TestPlayer::default(),
            enemy_agent: None,
        }
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

    /// Sets the enemy to be an AI agent instead of a human player.
    pub fn enemy_agent(mut self, agent: GameAI) -> Self {
        self.enemy_agent = Some(agent);
        self
    }

    /// Sets the seed for deterministic random number generation.
    pub fn seed(mut self, seed: u64) -> Self {
        self.session.seed = Some(seed);
        self
    }

    /// Sets the dreamwell card list for the session.
    pub fn with_dreamwell(mut self, id: DreamwellCardIdList) -> Self {
        self.session = self.session.with_dreamwell(id);
        self
    }

    /// Connects to the rules engine, returning the session struct. Moves all
    /// player hands into their decks.
    ///
    /// Applies debug commands to populate the current battle state.
    pub fn connect(mut self) -> TestSession {
        let opponent = self
            .enemy_agent
            .map(PlayerType::Agent)
            .or(Some(PlayerType::User(self.session.enemy_id)));
        self.session.connect_with_opponent(opponent);
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
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::MoveHandToDeck {
            player: PlayerName::One,
        }));
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::MoveHandToDeck {
            player: PlayerName::Two,
        }));
    }

    fn apply_player_configuration(&mut self, player: PlayerName, config: &TestPlayer) {
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::SetPoints {
            player,
            points: config.points,
        }));
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::SetEnergy {
            player,
            energy: config.energy,
        }));
        self.session.perform_user_action(BattleAction::Debug(
            DebugBattleAction::SetProducedEnergy { player, energy: config.produced_energy },
        ));
        self.session.perform_user_action(BattleAction::Debug(DebugBattleAction::SetSparkBonus {
            player,
            spark: config.spark_bonus,
        }));
    }
}
