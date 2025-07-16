use std::collections::HashMap;

use core_data::numerics::Energy;
use display_data::battle_view::{BattlePreviewView, DisplayPlayer};
use display_data::card_view::{CardView, ClientCardId, RevealedCardView};
use display_data::object_position::Position;

use crate::client::test_client_card_list::TestClientCardList;

#[derive(Default)]
pub struct TestClientCards {
    pub card_map: HashMap<ClientCardId, TestClientCard>,
}

pub struct TestClientCard {
    pub id: ClientCardId,
    pub view: CardView,
}

impl TestClientCards {
    /// Get all cards in a specific position
    pub fn cards_at_position(&self, position: &Position) -> TestClientCardList<'_> {
        let cards = self
            .card_map
            .values()
            .filter(|card| &card.view.position.position == position)
            .collect();
        TestClientCardList::new(cards)
    }

    /// Get all cards in the user's hand
    pub fn user_hand(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InHand(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's hand
    pub fn enemy_hand(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InHand(DisplayPlayer::Enemy))
    }

    /// Get all cards on the user's battlefield
    pub fn user_battlefield(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::OnBattlefield(DisplayPlayer::User))
    }

    /// Get all cards on the enemy's battlefield
    pub fn enemy_battlefield(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::OnBattlefield(DisplayPlayer::Enemy))
    }

    /// Get all cards in the user's void
    pub fn user_void(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InVoid(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's void
    pub fn enemy_void(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InVoid(DisplayPlayer::Enemy))
    }

    /// Get all cards in the user's banished zone
    pub fn user_banished(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InBanished(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's banished zone
    pub fn enemy_banished(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InBanished(DisplayPlayer::Enemy))
    }

    /// Get all cards in the user's deck
    pub fn user_deck(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InDeck(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's deck
    pub fn enemy_deck(&self) -> TestClientCardList<'_> {
        self.cards_at_position(&Position::InDeck(DisplayPlayer::Enemy))
    }

    /// Get all cards on the stack
    pub fn stack_cards(&self) -> TestClientCardList<'_> {
        let cards = self
            .card_map
            .values()
            .filter(|card| matches!(&card.view.position.position, Position::OnStack(_)))
            .collect();
        TestClientCardList::new(cards)
    }

    /// Get a card by its ID.
    ///
    /// Panics if the card is not found.
    pub fn get(&self, id: &ClientCardId) -> &TestClientCard {
        self.card_map.get(id).unwrap_or_else(|| panic!("Card not found: {id}"))
    }

    /// Get the revealed card view for a card.
    ///
    /// Panics if the card is not found or is not revealed.
    pub fn get_revealed(&self, id: &ClientCardId) -> &RevealedCardView {
        let Some(revealed) = &self.get(id).view.revealed else {
            panic!("Card not found: {id}");
        };
        revealed
    }

    pub fn get_play_effect_preview(&self, id: &ClientCardId) -> &BattlePreviewView {
        let Some(revealed) = &self.get(id).view.revealed else {
            panic!("Card not found: {id}");
        };
        revealed.actions.play_effect_preview.as_ref().expect("Card has no play effect preview")
    }

    /// Get the cost of a card.
    ///
    /// Panics if the card is not found or has no cost.
    pub fn get_cost(&self, id: &ClientCardId) -> Energy {
        self.get_revealed(id).cost.expect("Card has no cost")
    }
}
