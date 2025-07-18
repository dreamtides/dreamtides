use action_data::game_action_data::GameAction;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::CommandSequence;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, DebugConfiguration, Metadata, PerformActionRequest,
};
use rules_engine::engine;
use state_provider::test_state_provider::TestStateProvider;
use uuid::Uuid;

use crate::client::test_client::TestClient;

pub struct TestSession {
    pub state_provider: TestStateProvider,
    pub user_id: UserId,
    pub enemy_id: UserId,
    pub battle_id: Option<BattleId>,
    pub user_client: TestClient,
    pub enemy_client: TestClient,
    pub last_user_response_version: Option<Uuid>,
    pub last_enemy_response_version: Option<Uuid>,
    pub seed: Option<u64>,
    pub last_user_commands: Option<CommandSequence>,
    pub last_enemy_commands: Option<CommandSequence>,
}

impl Default for TestSession {
    fn default() -> Self {
        Self::new()
    }
}

impl TestSession {
    pub fn new() -> Self {
        Self {
            state_provider: TestStateProvider::new(),
            user_id: UserId(Uuid::new_v4()),
            enemy_id: UserId(Uuid::new_v4()),
            battle_id: None,
            user_client: TestClient::default(),
            enemy_client: TestClient::default(),
            last_user_response_version: None,
            last_enemy_response_version: None,
            seed: None,
            last_user_commands: None,
            last_enemy_commands: None,
        }
    }

    pub fn client(&self, name: DisplayPlayer) -> &TestClient {
        match name {
            DisplayPlayer::User => &self.user_client,
            DisplayPlayer::Enemy => &self.enemy_client,
        }
    }

    /// Set the seed for deterministic random number generation
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Connects to the rules engine and applies the commands to the client.
    pub fn connect(&mut self) -> ConnectResponse {
        self.connect_with_opponent(Some(PlayerType::User(self.enemy_id)))
    }

    /// Connects to the rules engine with a specific opponent configuration.
    pub fn connect_with_opponent(&mut self, opponent: Option<PlayerType>) -> ConnectResponse {
        let response = engine::connect_with_provider(
            self.state_provider.clone(),
            &ConnectRequest {
                metadata: self.metadata(),
                persistent_data_path: "".to_string(),
                vs_opponent: None,
                display_properties: None,
                debug_configuration: Some(DebugConfiguration {
                    enemy: opponent.clone(),
                    seed: self.seed.or(Some(314159265358979323)),
                }),
            },
            self.request_context(),
        );

        if let Some(battle_id) = response.metadata.battle_id {
            self.battle_id = Some(battle_id);
        }

        self.last_user_response_version = Some(response.response_version);
        self.user_client.apply_commands(response.commands.clone());

        // If opponent is not a user, don't try to connect as them
        if let Some(PlayerType::User(_)) = opponent {
            let enemy_response = engine::connect_with_provider(
                self.state_provider.clone(),
                &ConnectRequest {
                    metadata: self.enemy_metadata(),
                    persistent_data_path: "".to_string(),
                    vs_opponent: Some(self.user_id),
                    display_properties: None,
                    debug_configuration: None,
                },
                self.request_context(),
            );

            self.last_enemy_response_version = Some(enemy_response.response_version);
            self.enemy_client.apply_commands(enemy_response.commands);
        }

        response
    }

    /// Performs a Game Action as the user player.
    ///
    /// This function will call the rules engine to execute the provided action
    /// synchronously. It blocks until the action has been fully executed and
    /// returns after applying all commands to the client.
    pub fn perform_user_action(&mut self, action: impl Into<GameAction>) {
        self.perform_action_internal(
            action,
            self.metadata(),
            self.user_id,
            self.enemy_id,
            self.last_user_response_version,
        );
    }

    /// Performs a Game Action as the enemy player.
    ///
    /// See [Self::perform_user_action].
    pub fn perform_enemy_action(&mut self, action: impl Into<GameAction>) {
        self.perform_action_internal(
            action,
            self.enemy_metadata(),
            self.user_id,
            self.user_id,
            self.last_enemy_response_version,
        );
    }

    /// Performs a Game Action as a specific player.
    ///
    /// See [Self::perform_user_action].
    pub fn perform_player_action(&mut self, player: DisplayPlayer, action: impl Into<GameAction>) {
        match player {
            DisplayPlayer::User => self.perform_user_action(action),
            DisplayPlayer::Enemy => self.perform_enemy_action(action),
        }
    }

    /// Converts a DisplayPlayer to a PlayerName.
    pub fn to_player_name(&self, player: DisplayPlayer) -> PlayerName {
        match player {
            DisplayPlayer::User => PlayerName::One,
            DisplayPlayer::Enemy => PlayerName::Two,
        }
    }

    fn perform_action_internal(
        &mut self,
        action: impl Into<GameAction>,
        metadata: Metadata,
        save_file_id: UserId,
        opponent_id: UserId,
        last_response_version: Option<Uuid>,
    ) {
        let action = action.into();

        let request = PerformActionRequest {
            metadata,
            action,
            save_file_id: Some(save_file_id),
            last_response_version,
        };

        let result = engine::perform_action_blocking(
            self.state_provider.clone(),
            request,
            Some(opponent_id),
        );

        let mut user_commands = CommandSequence::default();
        let mut enemy_commands = CommandSequence::default();

        for poll_result in result.user_poll_results {
            if metadata.user_id == self.user_id {
                user_commands.groups.extend(poll_result.commands.groups.clone());
                self.user_client.apply_commands(poll_result.commands);
            } else {
                enemy_commands.groups.extend(poll_result.commands.groups.clone());
                self.enemy_client.apply_commands(poll_result.commands);
            }
        }

        for poll_result in result.enemy_poll_results {
            if opponent_id == self.user_id {
                user_commands.groups.extend(poll_result.commands.groups.clone());
                self.user_client.apply_commands(poll_result.commands);
            } else {
                enemy_commands.groups.extend(poll_result.commands.groups.clone());
                self.enemy_client.apply_commands(poll_result.commands);
            }
        }

        self.last_user_commands = Some(user_commands);
        self.last_enemy_commands = Some(enemy_commands);
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            user_id: self.user_id,
            battle_id: self.battle_id,
            request_id: None,
            integration_test_id: None,
        }
    }

    fn enemy_metadata(&self) -> Metadata {
        Metadata {
            user_id: self.enemy_id,
            battle_id: self.battle_id,
            request_id: None,
            integration_test_id: None,
        }
    }

    fn request_context(&self) -> RequestContext {
        RequestContext { logging_options: LoggingOptions::default() }
    }
}
