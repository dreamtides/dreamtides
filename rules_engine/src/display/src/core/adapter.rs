use battle_state::battle::card_id::{CardId, CardIdType};
use display_data::card_view::ClientCardId;
use display_data::command::GameObjectId;

/// Converts a [CardId] to a [ClientCardId].
pub fn client_card_id(card_id: CardId) -> ClientCardId {
    card_id.0.to_string()
}

pub fn card_game_object_id(id: impl CardIdType) -> GameObjectId {
    GameObjectId::CardId(client_card_id(id.card_id()))
}
