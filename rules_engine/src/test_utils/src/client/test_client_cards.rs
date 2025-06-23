use std::collections::HashMap;

use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{CardView, ClientCardId};
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
    pub fn cards_at_position(&self, position: &Position) -> TestClientCardList {
        let cards = self
            .card_map
            .values()
            .filter(|card| &card.view.position.position == position)
            .collect();
        TestClientCardList::new(cards)
    }

    /// Get all cards in the user's hand
    pub fn user_hand(&self) -> TestClientCardList {
        self.cards_at_position(&Position::InHand(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's hand
    pub fn enemy_hand(&self) -> TestClientCardList {
        self.cards_at_position(&Position::InHand(DisplayPlayer::Enemy))
    }

    /// Get all cards on the user's battlefield
    pub fn user_battlefield(&self) -> TestClientCardList {
        self.cards_at_position(&Position::OnBattlefield(DisplayPlayer::User))
    }

    /// Get all cards on the enemy's battlefield
    pub fn enemy_battlefield(&self) -> TestClientCardList {
        self.cards_at_position(&Position::OnBattlefield(DisplayPlayer::Enemy))
    }

    /// Get all cards in the user's void
    pub fn user_void(&self) -> TestClientCardList {
        self.cards_at_position(&Position::InVoid(DisplayPlayer::User))
    }

    /// Get all cards in the enemy's void
    pub fn enemy_void(&self) -> TestClientCardList {
        self.cards_at_position(&Position::InVoid(DisplayPlayer::Enemy))
    }

    /// Get all cards on the stack
    pub fn stack_cards(&self) -> TestClientCardList {
        let cards = self
            .card_map
            .values()
            .filter(|card| matches!(&card.view.position.position, Position::OnStack(_)))
            .collect();
        TestClientCardList::new(cards)
    }

    /// Get a card by its ID
    pub fn get(&self, id: &ClientCardId) -> Option<&TestClientCard> {
        self.card_map.get(id)
    }
}
