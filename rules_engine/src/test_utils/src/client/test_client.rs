use action_data::game_action_data::GameAction;
use core_data::display_types::AudioClipAddress;
use core_data::identifiers::BattleId;
use display_data::battle_view::{
    BattlePreviewState, BattlePreviewView, ButtonView, DisplayPlayer, InterfaceView,
};
use display_data::card_view::CardView;
use display_data::command::{
    Command, CommandSequence, DisplayArrow, GameMessageType, UpdateBattleCommand,
    UpdateQuestCommand,
};

use crate::client::test_client_cards::{TestClientCard, TestClientCards};
use crate::client::test_client_player::TestClientPlayer;
use crate::client::test_interface_view::TestInterfaceView;

/// Represents a user client connected to a test game
#[derive(Default)]
pub struct TestClient {
    pub cards: TestClientCards,
    /// A player's view of *their own* player state.
    pub me: TestClientPlayer,
    /// A player's view of *their opponent's* player state.
    pub opponent: TestClientPlayer,
    /// Current battle ID
    pub battle_id: Option<BattleId>,
    /// Current interface state (buttons, overlays, etc.)
    pub interface: TestInterfaceView,
    /// Current arrows displayed between cards
    pub arrows: Vec<DisplayArrow>,
    /// Current battle preview state
    pub preview: Option<BattlePreviewState>,
    /// Last played audio clip
    pub last_audio_clip: Option<AudioClipAddress>,
    /// Last displayed game message
    pub last_game_message: Option<GameMessageType>,
    /// Cards drawn by the user during the last draw command
    pub last_drawn_cards: Vec<CardView>,
}

impl TestClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_commands(&mut self, commands: CommandSequence) {
        for group in commands.groups {
            for command in group.commands {
                match command {
                    Command::UpdateBattle(update) => self.handle_update_battle(*update),
                    Command::UpdateQuest(update) => self.handle_update_quest(*update),
                    Command::Wait(_) => {}
                    Command::FireProjectile(_) => {}
                    Command::DissolveCard(_) => {}
                    Command::DisplayGameMessage(msg) => {
                        self.last_game_message = Some(msg);
                    }
                    Command::DisplayEffect(_) => {}
                    Command::PlayAudioClip(audio) => {
                        self.last_audio_clip = Some(audio.sound);
                    }
                    Command::MoveCardsWithCustomAnimation(draw) => {
                        self.last_drawn_cards = draw.cards.clone();
                    }
                    Command::DisplayJudgment(judgment) => {
                        if let Some(new_score) = judgment.new_score {
                            match judgment.player {
                                DisplayPlayer::User => {
                                    if let Some(ref mut view) = self.me.view {
                                        view.score = new_score;
                                    }
                                }
                                DisplayPlayer::Enemy => {
                                    if let Some(ref mut view) = self.opponent.view {
                                        view.score = new_score;
                                    }
                                }
                            }
                        }
                    }
                    Command::DisplayDreamwellActivation(activation) => match activation.player {
                        DisplayPlayer::User => {
                            if let Some(ref mut view) = self.me.view {
                                if let Some(energy) = activation.new_energy {
                                    view.energy = energy;
                                }
                                if let Some(produced) = activation.new_produced_energy {
                                    view.produced_energy = produced;
                                }
                            }
                        }
                        DisplayPlayer::Enemy => {
                            if let Some(ref mut view) = self.opponent.view {
                                if let Some(energy) = activation.new_energy {
                                    view.energy = energy;
                                }
                                if let Some(produced) = activation.new_produced_energy {
                                    view.produced_energy = produced;
                                }
                            }
                        }
                    },
                    Command::DisplayEnemyMessage(_) => {}
                    Command::PlayStudioAnimation(_) => {}
                    Command::SetCardTrail(_) => {}
                    Command::ShuffleVoidIntoDeck(_) => {}
                    Command::UpdateScreenOverlay(overlay) => {
                        if let Some(interface) = self.interface.view.as_mut() {
                            interface.screen_overlay = overlay.screen_overlay;
                        }
                    }
                }
            }
        }
    }

    fn handle_update_battle(&mut self, update: UpdateBattleCommand) {
        let battle = update.battle;

        self.battle_id = Some(battle.id);

        self.me.view = Some(battle.user);
        self.opponent.view = Some(battle.enemy);

        self.cards.card_map.clear();
        for card in battle.cards {
            self.cards
                .card_map
                .insert(card.id.clone(), TestClientCard { id: card.id.clone(), view: card });
        }

        self.interface = TestInterfaceView::new(Some(battle.interface));

        self.arrows = battle.arrows;

        self.preview = Some(battle.preview);

        if let Some(sound) = update.update_sound {
            self.last_audio_clip = Some(sound);
        }
    }

    fn handle_update_quest(&mut self, update: UpdateQuestCommand) {
        self.cards.card_map.clear();
        for card in update.quest.cards {
            self.cards
                .card_map
                .insert(card.id.clone(), TestClientCard { id: card.id.clone(), view: card });
        }

        self.interface = TestInterfaceView::new(Some(update.quest.interface));

        if let Some(sound) = update.update_sound {
            self.last_audio_clip = Some(sound);
        }
    }

    /// Get the current interface state
    pub fn interface(&self) -> &InterfaceView {
        self.interface.view.as_ref().expect("No interface present")
    }

    /// Get the primary action button
    pub fn primary_action_button(&self) -> &ButtonView {
        self.interface
            .view
            .as_ref()
            .expect("No interface present")
            .primary_action_button
            .as_ref()
            .expect("No primary action button present")
    }

    /// Get the secondary action button
    pub fn secondary_action_button(&self) -> &ButtonView {
        self.interface
            .view
            .as_ref()
            .expect("No interface present")
            .secondary_action_button
            .as_ref()
            .expect("No secondary action button present")
    }

    /// Check if the game has ended
    pub fn is_game_over(&self) -> bool {
        matches!(self.last_game_message, Some(GameMessageType::Victory | GameMessageType::Defeat))
    }

    /// Check if the user won
    pub fn user_won(&self) -> bool {
        matches!(self.last_game_message, Some(GameMessageType::Victory))
    }

    /// Get count of revealed cards in user's hand
    pub fn user_hand_size(&self) -> usize {
        self.cards.user_hand().iter().filter(|card| card.view.revealed.is_some()).count()
    }

    /// Get count of cards in enemy's hand (revealed or not)
    pub fn enemy_hand_size(&self) -> usize {
        self.cards.enemy_hand().len()
    }

    /// Check if any overlay is currently shown
    pub fn has_screen_overlay(&self) -> bool {
        self.interface.view.as_ref().and_then(|i| i.screen_overlay.as_ref()).is_some()
    }

    /// Get all legal actions currently available in the interface
    pub fn legal_actions(&self) -> Vec<GameAction> {
        let mut actions = Vec::new();

        // Collect actions from interface buttons
        if let Some(interface) = &self.interface.view {
            if let Some(button) = &interface.primary_action_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.secondary_action_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.increment_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.decrement_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.dev_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.undo_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
            if let Some(button) = &interface.bottom_right_button
                && let Some(action) = &button.action
            {
                actions.push(action.clone());
            }
        }

        // Collect actions from cards (sorted by card ID for deterministic order)
        let mut cards: Vec<_> = self.cards.card_map.values().collect();
        cards.sort_by_key(|card| &card.id);
        for card in cards {
            if let Some(revealed) = &card.view.revealed {
                if let Some(action) = &revealed.actions.can_play {
                    actions.push(action.clone());
                }
                if let Some(action) = &revealed.actions.on_click {
                    actions.push(action.clone());
                }
            }
        }

        actions
    }

    /// Get the active battle preview
    ///
    /// Panics if no preview is active.
    pub fn active_battle_preview(&self) -> &BattlePreviewView {
        if let Some(BattlePreviewState::Active(preview)) = &self.preview {
            preview
        } else {
            panic!("No active battle preview");
        }
    }
}
