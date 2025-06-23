use display_data::card_view::ClientCardId;

use crate::client::test_client_cards::TestClientCard;

pub struct TestClientCardList<'a> {
    pub cards: Vec<&'a TestClientCard>,
}

impl<'a> TestClientCardList<'a> {
    pub fn new(cards: Vec<&'a TestClientCard>) -> Self {
        Self { cards }
    }

    /// Returns true if the list contains a card with the given ID.
    pub fn contains(&self, card_id: &ClientCardId) -> bool {
        self.cards.iter().any(|card| &card.id == card_id)
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<&'a TestClientCard> {
        self.cards.iter()
    }

    pub fn get(&self, index: usize) -> Option<&&'a TestClientCard> {
        self.cards.get(index)
    }
}
