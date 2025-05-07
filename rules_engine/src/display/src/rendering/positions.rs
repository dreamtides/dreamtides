use battle_data_old::battle_cards::card_data::CardData;
use battle_data_old::battle_cards::zone::Zone;
use display_data::object_position::{ObjectPosition, Position, StackType};

use crate::core::response_builder::ResponseBuilder;

pub fn calculate(builder: &ResponseBuilder, card: &CardData) -> ObjectPosition {
    let player = builder.to_display_player(card.controller());
    let position = match card.zone {
        Zone::Hand => Position::InHand(player),
        Zone::Deck => Position::InDeck(player),
        Zone::Battlefield => Position::OnBattlefield(player),
        Zone::Stack => Position::OnStack(StackType::Default),
        Zone::Void => Position::InVoid(player),
        Zone::Banished => Position::InBanished(player),
    };

    for_card(card, position)
}

pub fn for_card(card: &CardData, position: Position) -> ObjectPosition {
    let sorting_key = card.object_id.0;
    ObjectPosition { position, sorting_key, sorting_sub_key: 0 }
}
