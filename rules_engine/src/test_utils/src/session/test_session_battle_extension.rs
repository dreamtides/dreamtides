use std::collections::HashSet;

use action_data::game_action_data::GameAction;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use display_data::battle_view::{ButtonView, DisplayPlayer};
use display_data::card_view::ClientCardId;
use display_data::command::Command;

use crate::session::test_session::TestSession;

pub struct TestPlayCard {
    pub name: CardName,
    pub target: Option<ClientCardId>,
}

impl TestPlayCard {
    pub fn new(name: CardName) -> Self {
        Self { name, target: None }
    }

    pub fn target(mut self, target: &ClientCardId) -> Self {
        self.target = Some(target.clone());
        self
    }
}

impl From<CardName> for TestPlayCard {
    fn from(name: CardName) -> Self {
        Self { name, target: None }
    }
}

/// Extension trait for TestSession to add battle-specific methods.
pub trait TestSessionBattleExtension {
    /// Creates and then plays a card according to a [TestPlayCard] description.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via debug actions. The card is then played from hand via the standard
    /// play card action, i.e. it must be legal to play the card at this time.
    ///
    /// Panics if the server returns an error for playing this card or if it
    /// cannot currently be played (e.g. due to insufficient energy).
    ///
    /// Returns the ID of the newly played card.
    fn create_and_play(
        &mut self,
        player: DisplayPlayer,
        card: impl Into<TestPlayCard>,
    ) -> ClientCardId;

    /// Plays a card from a player's hand via the standard play card action.
    ///
    /// Panics if the server returns an error for playing this card or if it
    /// cannot currently be played (e.g. due to insufficient energy).
    fn play_card_from_hand(&mut self, player: DisplayPlayer, card_id: &ClientCardId);

    /// Selects a card as a target via its 'on_click' action.
    ///
    /// Panics if the server returns an error for selecting this target or if
    /// the target is not valid for the card.
    fn select_target(&mut self, player: DisplayPlayer, target_id: &ClientCardId);

    /// Adds a card to a player's hand via debug actions, returning its card id.
    fn add_to_hand(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId;

    /// Adds a card to a player's battlefield via debug actions, returning its
    /// card id. This does not play the card or spend energy etc.
    fn add_to_battlefield(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId;

    fn activate_ability(
        &mut self,
        player: DisplayPlayer,
        character_id: &ClientCardId,
        ability_number: usize,
    );

    /// Takes the "end turn" action as the named `player`. Moves all cards from
    /// the opponent player's hand to their deck via debug actions.
    ///
    /// Intended to help bypass the effects of the opponent player drawing a
    /// card for their turn.
    fn end_turn_remove_opponent_hand(&mut self, player: DisplayPlayer);

    /// Clicks the primary button for the named `player` containing the given
    /// `label`.
    ///
    /// Panics if the server returns an error for clicking this button, if the
    /// label does not match, or if the button is disabled or not present.
    fn click_primary_button(&mut self, player: DisplayPlayer, containing: impl Into<String>);

    /// Clicks the secondary button for the named `player` containing the given
    /// `label`.
    ///
    /// Panics if the server returns an error for clicking this button, if the
    /// label does not match, or if the button is disabled or not present.
    fn click_secondary_button(&mut self, player: DisplayPlayer, containing: impl Into<String>);

    /// Clicks the increment button for the named `player`.
    ///
    /// Panics if the server returns an error for clicking this button or if the
    /// button is disabled or not present.
    fn click_increment_button(&mut self, player: DisplayPlayer);

    /// Clicks the decrement button for the named `player`.
    ///
    /// Panics if the server returns an error for clicking this button or if the
    /// button is disabled or not present.
    fn click_decrement_button(&mut self, player: DisplayPlayer);

    /// Finds a command of the specified type for a specific player.
    ///
    /// Returns the command if found, panics if not found or if multiple
    /// commands are found.
    fn find_command<T>(
        &self,
        player: DisplayPlayer,
        command_extractor: impl Fn(&Command) -> Option<&T>,
    ) -> &T;

    /// Finds all commands of the specified type for a specific player.
    ///
    /// Returns a vector of all matching commands.
    fn find_all_commands<T>(
        &self,
        player: DisplayPlayer,
        command_extractor: impl Fn(&Command) -> Option<&T>,
    ) -> Vec<&T>;
}

impl TestSessionBattleExtension for TestSession {
    fn create_and_play(
        &mut self,
        player: DisplayPlayer,
        card: impl Into<TestPlayCard>,
    ) -> ClientCardId {
        let card = card.into();

        let new_card_id = self.add_to_hand(player, card.name);
        self.play_card_from_hand(player, &new_card_id);

        if let Some(target) = card.target {
            self.select_target(player, &target);
        }

        new_card_id
    }

    fn play_card_from_hand(&mut self, player: DisplayPlayer, card_id: &ClientCardId) {
        let play_action = self
            .client(player)
            .cards
            .get_revealed(card_id)
            .actions
            .can_play
            .clone()
            .expect("Card cannot be played from hand");

        self.perform_player_action(player, play_action);
    }

    fn select_target(&mut self, player: DisplayPlayer, target_id: &ClientCardId) {
        let target_action =
            self.client(player).cards.get_revealed(target_id).actions.on_click.clone();

        if target_action.is_none() {
            let battlefield_count = self.client(player).cards.user_battlefield().len();
            if battlefield_count == 1 {
                panic!(
                    "Target card has no on_click action and there is only one card on the \
                     battlefield. The target might have been automatically selected."
                );
            }
        }

        let target_action = target_action.expect("Target card has no on_click action");
        self.perform_player_action(player, target_action);
    }

    fn add_to_hand(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId {
        let existing_hand_ids: HashSet<String> =
            self.client(player).cards.user_hand().iter().map(|c| c.id.clone()).collect();

        self.perform_player_action(
            player,
            GameAction::BattleAction(BattleAction::Debug(DebugBattleAction::AddCardToHand {
                player: self.to_player_name(player),
                card,
            })),
        );

        self.client(player)
            .cards
            .user_hand()
            .iter()
            .find(|c| !existing_hand_ids.contains(&c.id))
            .map(|c| c.id.clone())
            .expect("Failed to find newly added card in hand")
    }

    fn add_to_battlefield(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId {
        let existing_battlefield_ids: HashSet<String> =
            self.client(player).cards.user_battlefield().iter().map(|c| c.id.clone()).collect();

        self.perform_player_action(
            player,
            BattleAction::Debug(DebugBattleAction::AddCardToBattlefield {
                player: self.to_player_name(player),
                card,
            }),
        );

        self.client(player)
            .cards
            .user_battlefield()
            .iter()
            .find(|c| !existing_battlefield_ids.contains(&c.id))
            .map(|c| c.id.clone())
            .expect("Failed to find newly added card on battlefield")
    }

    fn activate_ability(
        &mut self,
        player: DisplayPlayer,
        character_id: &ClientCardId,
        ability_number: usize,
    ) {
        let token_card_id = format!("A{character_id}/{ability_number}");
        self.play_card_from_hand(player, &token_card_id);
    }

    fn end_turn_remove_opponent_hand(&mut self, player: DisplayPlayer) {
        let opponent = match player {
            DisplayPlayer::User => DisplayPlayer::Enemy,
            DisplayPlayer::Enemy => DisplayPlayer::User,
        };

        self.perform_player_action(player, GameAction::BattleAction(BattleAction::EndTurn));

        self.perform_player_action(player, DebugBattleAction::MoveHandToDeck {
            player: self.to_player_name(opponent),
        });
    }

    fn click_primary_button(&mut self, player: DisplayPlayer, containing: impl Into<String>) {
        let containing = containing.into();
        let primary_button = self.client(player).interface().primary_action_button.clone();
        click_button(self, player, primary_button, "primary action button", &containing);
    }

    fn click_secondary_button(&mut self, player: DisplayPlayer, containing: impl Into<String>) {
        let containing = containing.into();
        let secondary_button = self.client(player).interface().secondary_action_button.clone();
        click_button(self, player, secondary_button, "secondary action button", &containing);
    }

    fn click_increment_button(&mut self, player: DisplayPlayer) {
        let increment_button = self.client(player).interface().increment_button.clone();
        click_button(self, player, increment_button, "increment button", "");
    }

    fn click_decrement_button(&mut self, player: DisplayPlayer) {
        let decrement_button = self.client(player).interface().decrement_button.clone();
        click_button(self, player, decrement_button, "decrement button", "");
    }

    fn find_command<T>(
        &self,
        player: DisplayPlayer,
        command_extractor: impl Fn(&Command) -> Option<&T>,
    ) -> &T {
        let commands = match player {
            DisplayPlayer::User => self.last_user_commands.as_ref(),
            DisplayPlayer::Enemy => self.last_enemy_commands.as_ref(),
        };

        let commands = commands.expect("No commands found for player");

        let mut matching_commands: Vec<&T> = commands
            .groups
            .iter()
            .flat_map(|group| &group.commands)
            .filter_map(command_extractor)
            .collect();

        match matching_commands.len() {
            0 => panic!("Player command not found in last commands"),
            1 => matching_commands.remove(0),
            count => panic!("Found {count} matching player commands, expected exactly 1"),
        }
    }

    fn find_all_commands<T>(
        &self,
        player: DisplayPlayer,
        command_extractor: impl Fn(&Command) -> Option<&T>,
    ) -> Vec<&T> {
        let commands = match player {
            DisplayPlayer::User => self.last_user_commands.as_ref(),
            DisplayPlayer::Enemy => self.last_enemy_commands.as_ref(),
        };

        let commands = commands.expect("No commands found for player");

        commands
            .groups
            .iter()
            .flat_map(|group| &group.commands)
            .filter_map(command_extractor)
            .collect()
    }
}

fn click_button(
    session: &mut TestSession,
    player: DisplayPlayer,
    button: Option<ButtonView>,
    button_name: &str,
    containing: &str,
) {
    let button = button.unwrap_or_else(|| panic!("No {button_name} present"));

    if !button.label.contains(containing) {
        panic!(
            "{} label mismatch: expected '{}' to contain '{}'",
            button_name, button.label, containing
        );
    }

    let action = button.action.unwrap_or_else(|| panic!("{button_name} is disabled"));

    session.perform_player_action(player, action);
}
