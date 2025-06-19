use std::collections::HashMap;

use display_data::card_view::{CardView, ClientCardId};

#[derive(Default)]
pub struct TestClientCards {
    pub card_map: HashMap<ClientCardId, TestClientCard>,
}

pub struct TestClientCard {
    pub id: ClientCardId,
    pub view: CardView,
}
