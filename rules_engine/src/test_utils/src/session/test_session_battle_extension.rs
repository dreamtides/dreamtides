use std::collections::HashSet;

use action_data::game_action_data::GameAction;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use bon::Builder;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::ClientCardId;

use crate::session::test_session::TestSession;

#[derive(Builder)]
pub struct TestPlayCard {
    pub name: CardName,
    #[builder(into)]
    pub target: Option<ClientCardId>,
    #[builder(into)]
    pub as_player: Option<DisplayPlayer>,
}

impl From<CardName> for TestPlayCard {
    fn from(name: CardName) -> Self {
        Self { name, target: None, as_player: None }
    }
}

impl TestPlayCard {
    pub fn player(&self) -> DisplayPlayer {
        self.as_player.unwrap_or(DisplayPlayer::User)
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
    fn create_and_play(&mut self, card: impl Into<TestPlayCard>) -> ClientCardId;

    /// Adds a card to a player's hand via debug actions, returning its card id.
    fn add_to_hand(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId;

    /// Adds a card to a player's battlefield via debug actions, returning its
    /// card id. This does not play the card or spend energy etc.
    fn add_to_battlefield(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId;
}

impl TestSessionBattleExtension for TestSession {
    fn create_and_play(&mut self, card: impl Into<TestPlayCard>) -> ClientCardId {
        let card = card.into();
        let player = card.player();

        let new_card_id = self.add_to_hand(player, card.name);

        let play_action = self
            .client(player)
            .cards
            .get(&new_card_id)
            .and_then(|card| card.view.revealed.as_ref())
            .and_then(|revealed| revealed.actions.can_play.clone())
            .expect("Card cannot be played from hand");

        self.perform_player_action(player, play_action);

        if let Some(target) = card.target {
            let target_action = self
                .client(player)
                .cards
                .get(&target)
                .and_then(|card| card.view.revealed.as_ref())
                .and_then(|revealed| revealed.actions.on_click.clone());

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

        new_card_id
    }

    fn add_to_hand(&mut self, player: DisplayPlayer, card: CardName) -> ClientCardId {
        let existing_hand_ids: HashSet<String> =
            self.client(player).cards.user_hand().iter().map(|c| c.id.clone()).collect();

        self.perform_player_action(
            player,
            GameAction::BattleAction(BattleAction::Debug(DebugBattleAction::AddCardToHand(
                self.to_player_name(player),
                card,
            ))),
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
            BattleAction::Debug(DebugBattleAction::AddCardToBattlefield(
                self.to_player_name(player),
                card,
            )),
        );

        self.client(player)
            .cards
            .user_battlefield()
            .iter()
            .find(|c| !existing_battlefield_ids.contains(&c.id))
            .map(|c| c.id.clone())
            .expect("Failed to find newly added card on battlefield")
    }
}
