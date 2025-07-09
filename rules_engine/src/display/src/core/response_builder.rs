use std::sync::Arc;

use battle_state::battle::battle_animation::TriggerAnimation;
use battle_state::battle::card_id::ActivatedAbilityId;
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display_data::battle_view::{BattleView, DisplayPlayer};
use display_data::command::{Command, CommandSequence, ParallelCommandGroup, UpdateBattleCommand};
use state_provider::display_state_provider::{DisplayState, DisplayStateProvider};

/// Primary builder used to render game state.
pub struct ResponseBuilder {
    /// Player for whom we are rendering
    player: PlayerName,

    /// User ID for whom we are rendering
    user_id: Option<UserId>,

    /// State provider for managing display state
    provider: Option<Arc<dyn DisplayStateProvider>>,

    /// Whether to animate the commands.
    animate: bool,

    /// Commands to be executed as part of the response.
    commands: CommandSequence,

    /// Whether this is an intermediate update for animation purposes.
    for_animation: bool,

    /// Commands to run in parallel with the next battle view update.
    pending_commands: Vec<Command>,

    /// Triggers which are currently active.
    active_triggers: Vec<TriggerAnimation>,

    /// Ability which is currently being activated.
    currently_activating_ability: Option<CurrentlyActivatingAbility>,
}

#[derive(Clone, Debug, Copy)]
pub struct CurrentlyActivatingAbility {
    pub player: PlayerName,
    pub activated_ability_id: ActivatedAbilityId,
}

impl ResponseBuilder {
    pub fn new(player: PlayerName, animate: bool) -> Self {
        Self {
            player,
            user_id: None,
            provider: None,
            animate,
            commands: CommandSequence::default(),
            for_animation: false,
            pending_commands: Vec::new(),
            active_triggers: Vec::new(),
            currently_activating_ability: None,
        }
    }

    pub fn with_state_provider(
        player: PlayerName,
        user_id: UserId,
        provider: impl DisplayStateProvider + 'static,
        animate: bool,
    ) -> Self {
        Self {
            player,
            user_id: Some(user_id),
            provider: Some(Arc::new(provider)),
            animate,
            commands: CommandSequence::default(),
            for_animation: false,
            pending_commands: Vec::new(),
            active_triggers: Vec::new(),
            currently_activating_ability: None,
        }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.groups.push(ParallelCommandGroup { commands: vec![command] });
    }

    pub fn push_battle_view(&mut self, view: BattleView) {
        let mut commands = vec![Command::UpdateBattle(Box::new(UpdateBattleCommand {
            battle: view,
            update_sound: None,
        }))];
        commands.append(&mut self.pending_commands);
        self.commands.groups.push(ParallelCommandGroup { commands });
    }

    /// Appends all of the groups from a [CommandSequence] to the response.
    pub fn extend(&mut self, commands: CommandSequence) {
        self.commands.groups.extend(commands.groups);
    }

    /// Optional equivalent of [Self::extend].
    pub fn extend_optional(&mut self, commands: Option<CommandSequence>) {
        if let Some(commands) = commands {
            self.extend(commands);
        }
    }

    /// Adds a command to run in parallel with the next `UpdateBattleCommand`
    /// that is received via [Self::push_battle_view].
    pub fn run_with_next_battle_view(&mut self, command: Command) {
        self.pending_commands.push(command);
    }

    pub fn should_animate(&self) -> bool {
        self.animate
    }

    pub fn display_for_player(&self) -> PlayerName {
        self.player
    }

    pub fn act_for_player(&self) -> PlayerName {
        self.player
    }

    pub fn commands(self) -> CommandSequence {
        self.commands
    }

    pub fn set_for_animation(&mut self, for_animation: bool) {
        self.for_animation = for_animation;
    }

    pub fn is_for_animation(&self) -> bool {
        self.for_animation
    }

    pub fn to_display_player(&self, player: PlayerName) -> DisplayPlayer {
        if self.player == player { DisplayPlayer::User } else { DisplayPlayer::Enemy }
    }

    pub fn get_display_state(&self) -> DisplayState {
        if let (Some(provider), Some(user_id)) = (&self.provider, &self.user_id) {
            provider.get_display_state(*user_id)
        } else {
            DisplayState::default()
        }
    }

    pub fn set_display_state(&self, state: DisplayState) {
        if let (Some(provider), Some(user_id)) = (&self.provider, &self.user_id) {
            provider.set_display_state(*user_id, state);
        }
    }

    pub fn update_display_state<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut DisplayState),
    {
        let mut state = self.get_display_state();
        update_fn(&mut state);
        self.set_display_state(state);
    }

    pub fn set_active_triggers(&mut self, triggers: Vec<TriggerAnimation>) {
        self.active_triggers = triggers;
    }

    pub fn active_triggers(&self) -> &[TriggerAnimation] {
        &self.active_triggers
    }

    pub fn set_currently_activating_ability(
        &mut self,
        currently_activating_ability: CurrentlyActivatingAbility,
    ) {
        self.currently_activating_ability = Some(currently_activating_ability);
    }

    pub fn currently_activating_ability(&self) -> Option<CurrentlyActivatingAbility> {
        self.currently_activating_ability
    }
}
