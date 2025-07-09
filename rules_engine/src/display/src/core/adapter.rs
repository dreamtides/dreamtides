use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::stack_card_state::StackItemId;
use display_data::card_view::ClientCardId;
use display_data::command::GameObjectId;

/// Converts a [CardId] to a [ClientCardId].
pub fn client_card_id(card_id: CardId) -> ClientCardId {
    card_id.0.to_string()
}

pub fn stack_item_client_card_id(item: impl Into<StackItemId>) -> ClientCardId {
    match item.into() {
        StackItemId::Card(card_id) => client_card_id(card_id.card_id()),
        StackItemId::ActivatedAbility(ability_id) => {
            format!("A{}/{}", ability_id.character_id.0.0, ability_id.ability_number.0)
        }
    }
}

pub fn card_game_object_id(id: impl CardIdType) -> GameObjectId {
    GameObjectId::CardId(client_card_id(id.card_id()))
}

pub fn stack_item_game_object_id(item: impl Into<StackItemId>) -> GameObjectId {
    GameObjectId::CardId(stack_item_client_card_id(item))
}
