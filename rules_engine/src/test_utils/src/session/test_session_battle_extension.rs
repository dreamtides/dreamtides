use std::collections::HashSet;

use action_data::game_action_data::GameAction;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;

use crate::session::test_session::TestSession;

/// Extension trait for TestSession to add battle-specific methods.
pub trait TestSessionBattleExtension {
    /// Creates and then plays a named card as the user player.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via debug actions. The card is then played from hand via the standard
    /// play card action.
    ///
    /// Panics if the server returns an error for playing this card or if it
    /// cannot currently be played (e.g. due to insufficient energy).
    fn create_and_play(&mut self, card: CardName);
}

impl TestSessionBattleExtension for TestSession {
    fn create_and_play(&mut self, card: CardName) {
        let existing_hand_ids: HashSet<String> =
            self.client.cards.user_hand().iter().map(|c| c.id.clone()).collect();

        self.perform_action(GameAction::BattleAction(BattleAction::Debug(
            DebugBattleAction::AddCardToHand(PlayerName::One, card),
        )));

        let new_card_id = self
            .client
            .cards
            .user_hand()
            .iter()
            .find(|c| !existing_hand_ids.contains(&c.id))
            .map(|c| c.id.clone())
            .expect("Failed to find newly added card in hand");

        let play_action = self
            .client
            .cards
            .get(&new_card_id)
            .and_then(|card| card.view.revealed.as_ref())
            .and_then(|revealed| revealed.actions.can_play.clone())
            .expect("Card cannot be played from hand");

        self.perform_action(play_action);
    }
}
